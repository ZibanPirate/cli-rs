use cli_man::{Cli, CliInstaller};
use std::path::PathBuf;

struct MockInstaller {
    name: String,
    version: String,
}

impl CliInstaller for MockInstaller {
    fn name(&self) -> &str {
        &self.name
    }
    fn version(&self) -> &str {
        &self.version
    }
    fn install(self) -> impl Cli {
        MockCli {
            bin_path: self.bin_path(),
        }
    }
}

struct MockCli {
    bin_path: PathBuf,
}

impl Cli for MockCli {
    fn bin_path(&self) -> PathBuf {
        self.bin_path.clone()
    }
}

#[test]
fn test_binary_name() {
    let installer = MockInstaller {
        name: "testcli".to_string(),
        version: "1.0.0".to_string(),
    };
    #[cfg(not(target_os = "windows"))]
    assert_eq!(installer.binary_name(), "testcli");
    #[cfg(target_os = "windows")]
    assert_eq!(installer.binary_name(), "testcli.exe");
}

#[test]
fn test_bin_path() {
    let installer = MockInstaller {
        name: "testcli".to_string(),
        version: "1.0.0".to_string(),
    };
    let home = std::env::var("HOME").unwrap();
    #[cfg(not(target_os = "windows"))]
    assert_eq!(
        installer.bin_path(),
        PathBuf::from(format!("{}/.cli-man/bin/testcli/1.0.0/testcli", home))
    );
    #[cfg(target_os = "windows")]
    assert_eq!(
        installer.bin_path(),
        PathBuf::from(format!("{}/.cli-man/bin/testcli/1.0.0/testcli.exe", home))
    );
}

#[test]
fn test_uninstall_nonexistent() {
    let cli = MockCli {
        bin_path: PathBuf::from("/tmp/cli-man-test/testcli/1.0.0/testcli"),
    };
    // Should not panic if directory doesn't exist
    cli.uninstall();
}

#[test]
fn test_uninstall_existing_dir() {
    use std::fs;
    use std::io::Write;

    // Create the full directory structure
    let version_dir = "./tmp/cli-man-test/testcli/1.0.0";
    let bin_path = format!("{}/testcli", version_dir);
    let tool_dir = "./tmp/cli-man-test/testcli"; // This is what actually gets removed

    // Create the directory and a dummy file
    fs::create_dir_all(version_dir).unwrap();
    let mut file = fs::File::create(&bin_path).unwrap();
    writeln!(file, "dummy").unwrap();

    let cli = MockCli {
        bin_path: PathBuf::from(&bin_path),
    };

    // Directory should exist before uninstall
    assert!(std::path::Path::new(tool_dir).exists());

    cli.uninstall();

    // Directory should not exist after uninstall
    assert!(!std::path::Path::new(tool_dir).exists());
}
