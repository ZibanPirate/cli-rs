use std::path::PathBuf;

pub fn download_and_install(url: &str, bin_path: &PathBuf) {
    let bin_dir = bin_path.parent().expect("Failed to get parent directory");
    if !bin_dir.exists() {
        std::fs::create_dir_all(bin_dir).expect("Failed to create directory");
    }

    let client = reqwest::blocking::Client::new();

    println!("Downloading from {}", url);
    let response = client.get(url).send().expect("Failed to download");

    if !response.status().is_success() {
        panic!("Failed to download: HTTP {}", response.status());
    }

    let content = response.bytes().expect("Failed to read response");

    if url.ends_with(".zip") {
        let reader = std::io::Cursor::new(content);
        let mut archive = zip::read::ZipArchive::new(reader).expect("Failed to read zip archive");
        archive.extract(bin_dir).expect("Failed to extract zip");
    } else if url.ends_with(".tar.gz") {
        todo!("Handle tar.gz files");
    } else {
        std::fs::write(bin_path, content).expect("Failed to write to file");
    }

    // Make the file executable on Unix-like systems
    #[cfg(not(target_os = "windows"))]
    {
        use std::{fs, os::unix::fs::PermissionsExt};

        let mut perms = fs::metadata(bin_path)
            .expect("Failed to get metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(bin_path, perms).expect("Failed to set permissions");
        println!("Permission applied to {}", bin_path.display());
    }
}
