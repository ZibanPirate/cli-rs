use dotenv::dotenv;
use std::{
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
    // Execute cmd with real-time output forwarding
    let mut child = Command::new(cmd.into())
        .current_dir(get_cli_run_cwd())
        .args(args.into_iter().map(Into::into).collect::<Vec<String>>())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

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
    let status = child.wait().expect("Failed to wait on child process");

    // Wait for the output forwarding to complete
    stdout_thread.join().expect("Failed to join stdout thread");
    stderr_thread.join().expect("Failed to join stderr thread");

    let code = status.code().unwrap_or(1);

    if code != 0 {
        std::process::exit(code);
    }
}
