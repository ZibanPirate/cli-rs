use crate::{Cli, CliInstaller, downloader::download_and_install};

pub struct Terraform;

impl Terraform {
    pub fn new_installer(version: impl Into<String>) -> TerraformInstaller {
        TerraformInstaller {
            version: version.into(),
        }
    }
}

pub struct TerraformInstaller {
    pub version: String,
}

impl CliInstaller for TerraformInstaller {
    fn name(&self) -> &str {
        "terraform"
    }

    fn install(self) -> impl Cli {
        if (self.bin_path()).exists() {
            return TerraformCli(self);
        }

        println!("Installing {}@{}", self.name(), self.version());

        // Determine OS and architecture
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else {
            "linux"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "amd64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "386"
        };

        let url = format!(
            "https://releases.hashicorp.com/terraform/{}/terraform_{}_{}_{}.zip",
            self.version, self.version, os, arch
        );

        download_and_install(&url, &self.bin_path());

        println!("Installed {}@{}", self.name(), self.version());

        TerraformCli(self)
    }

    fn version(&self) -> &str {
        &self.version
    }
}

pub struct TerraformCli(TerraformInstaller);

impl Cli for TerraformCli {
    fn bin_path(&self) -> std::path::PathBuf {
        self.0.bin_path()
    }
}
