use std::path::PathBuf;
use std::process::{Child, Command};

#[derive(Debug)]
pub struct IPykernel {
    process: Child,
    pub connection_file: PathBuf,
}

impl Default for IPykernel {
    fn default() -> Self {
        Self::new()
    }
}

impl IPykernel {
    pub fn new() -> Self {
        // write connection file at /tmp/kernel-sidecar-{uuid}.json
        let mut file_path = std::env::temp_dir();
        file_path.push(format!("kernel-sidecar-{}.json", uuid::Uuid::new_v4()));
        let connection_file = file_path;
        let process = Command::new("python")
            .arg("-m")
            .arg("ipykernel_launcher")
            .arg("-f")
            .arg(&connection_file)
            .spawn()
            .expect("Failed to start IPykernel");
        IPykernel {
            process,
            connection_file,
        }
    }

    pub async fn wait_for_file(&self) {
        loop {
            if self.connection_file.exists() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        }
    }
}

impl Drop for IPykernel {
    fn drop(&mut self) {
        self.process.kill().expect("Failed to kill IPykernel");
        self.connection_file
            .as_path()
            .to_owned()
            .into_os_string()
            .into_string()
            .expect("Failed to convert connection_file to string");
        std::fs::remove_file(&self.connection_file).expect("Failed to remove connection_file");
    }
}
