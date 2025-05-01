use dotenv::dotenv;
use std::{
    collections::HashMap,
    env, fs,
    io::{self, BufReader, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};

pub fn get_cli_run_cwd() -> PathBuf {
    dotenv().ok();
    let (_, cli_run_cwd) = env::vars()
        .find(|(key, _)| key == "CLI_RUN_CWD")
        .unwrap_or_else(|| {
            (
                "".to_string(),
                env::current_dir().unwrap().to_str().unwrap().to_string(),
            )
        });

    let cli_run_cwd = cli_run_cwd.replace("~", &env::var("HOME").unwrap());
    fs::canonicalize(cli_run_cwd).unwrap()
}

pub fn cli_run(cmd: impl Into<PathBuf>, args: Vec<impl Into<String>>) {
    let mut command = Command::new(cmd.into());
    command.current_dir(get_cli_run_cwd());
    command.envs(env::vars());
    command.args(args.into_iter().map(Into::into).collect::<Vec<String>>());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if let Err(e) = run_cmd_and_stream_output(&mut command) {
        eprintln!("Error running command: {}", e);
        std::process::exit(1);
    }
}

fn run_cmd_and_stream_output(cmd: &mut Command) -> Result<(), io::Error> {
    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Thread for forwarding stdout in real-time
    let stdout_thread = thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut buffer = [0; 1024];

        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 {
                break;
            }
            io::stdout()
                .write_all(&buffer[0..n])
                .expect("Failed to write to stdout");
            io::stdout().flush().expect("Failed to flush stdout"); // Ensure output is displayed immediately
        }
    });

    // Thread for forwarding stderr in real-time
    let stderr_thread = thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut buffer = [0; 1024];

        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 {
                break;
            }
            io::stderr().write_all(&buffer[0..n]).unwrap();
            io::stderr().flush().unwrap(); // Ensure output is displayed immediately
        }
    });

    // Wait for the process to complete
    let status = child.wait()?;

    // Wait for the output forwarding to complete
    stdout_thread.join().expect("Failed to join stdout thread");
    stderr_thread.join().expect("Failed to join stderr thread");

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Command failed with exit code: {}", status),
        ));
    }

    Ok(())
}

pub struct CliRun {
    cwd: PathBuf,
    extra_envs: HashMap<String, String>,
}

impl CliRun {
    pub fn new() -> CliRun {
        let cwd = get_cli_run_cwd();
        let extra_envs = HashMap::new();
        CliRun { cwd, extra_envs }
    }

    pub fn with_relative_cwd(self, cwd: impl Into<PathBuf>) -> CliRun {
        let cwd = self.cwd.join(cwd.into());
        let cwd = fs::canonicalize(cwd).unwrap();

        CliRun { cwd, ..self }
    }

    pub fn with_extra_envs(self, extra_envs: HashMap<String, String>) -> CliRun {
        CliRun { extra_envs, ..self }
    }

    pub fn run(&self, cmd: impl Into<PathBuf>, args: Vec<impl Into<String>>) {
        let mut command = Command::new(cmd.into());
        command.current_dir(&self.cwd);
        command.envs(&self.extra_envs);
        command.args(args.into_iter().map(Into::into).collect::<Vec<String>>());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        if let Err(e) = run_cmd_and_stream_output(&mut command) {
            eprintln!("Error running command: {}", e);
            std::process::exit(1);
        }
    }
}
