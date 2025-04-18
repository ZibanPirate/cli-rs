mod downloader;
pub mod terraform;

use std::path::PathBuf;
pub use terraform::*;

#[cfg(target_os = "windows")]
const WINDOWS_SUFFIX: &str = ".exe";

pub trait CliInstaller {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn binary_name(&self) -> String {
        #[cfg(target_os = "windows")]
        return format!("{}{}", self.name(), WINDOWS_SUFFIX);
        #[cfg(not(target_os = "windows"))]
        return self.name().to_string();
    }
    fn bin_path(&self) -> PathBuf {
        let home = if cfg!(windows) {
            std::env::var("USERPROFILE").unwrap()
        } else {
            std::env::var("HOME").unwrap()
        };

        PathBuf::from(format!(
            "{}/.cli-man/bin/{}/{}/{}",
            home,
            self.name(),
            self.version(),
            self.binary_name()
        ))
    }

    /// Installs the CLI tool into `~/.cli-man/bin/{tool}/{version}/{binary(.exe)}`
    fn install(self) -> impl Cli;
}

pub trait Cli {
    fn bin_path(&self) -> PathBuf;
    fn uninstall(&self) {
        let bin_path = self.bin_path();
        let bin_dir = bin_path.parent().unwrap().parent().unwrap();
        println!("Uninstalling binary directory: {}", bin_dir.display());
        if bin_dir.exists() {
            std::fs::remove_dir_all(bin_dir).expect("Failed to uninstall the binary directory");
            println!("Binary directory uninstalled successfully.");
        } else {
            println!("Binary directory not found: {}", bin_dir.display());
        }
    }
}
