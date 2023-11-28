use crate::jupyter::connection_file::ConnectionInfo;
use std::path::PathBuf;
use std::process::{Child, Command};

#[derive(Debug)]
pub struct JupyterKernel {
    process: Child,
    pub connection_info: ConnectionInfo,
    pub connection_file: PathBuf,
}

impl JupyterKernel {
    fn start_process(cmd: Vec<&str>, silent: bool) -> Child {
        Command::new(cmd[0])
            .args(&cmd[1..])
            .stdout(if silent {
                std::process::Stdio::null()
            } else {
                std::process::Stdio::inherit()
            })
            .spawn()
            .expect("Failed to start Jupyter Kernel")
    }

    // start an ipykernel process. If set to silent, stdout redirects to /dev/null
    pub fn ipython(silent: bool) -> Self {
        let kernel_name = "ipykernel".to_string();
        let connection_info = ConnectionInfo::new(Some(kernel_name)).unwrap();
        let file_path = connection_info.to_temp_file().unwrap();
        let cmd = vec![
            "python",
            "-m",
            "ipykernel_launcher",
            "-f",
            file_path.to_str().unwrap(),
        ];
        let process = Self::start_process(cmd, silent);
        Self {
            process,
            connection_info,
            connection_file: file_path,
        }
    }

    // start a Rust (evcxr) kernel.
    pub fn evcxr(silent: bool) -> Self {
        let kernel_name = "evcxr".to_string();
        let connection_info = ConnectionInfo::new(Some(kernel_name)).unwrap();
        let file_path = connection_info.to_temp_file().unwrap();
        let cmd = vec![
            "evcxr_jupyter",
            "--control_file",
            file_path.to_str().unwrap(),
        ];
        let process = Self::start_process(cmd, silent);
        Self {
            process,
            connection_info,
            connection_file: file_path,
        }
    }
}

impl Drop for JupyterKernel {
    fn drop(&mut self) {
        self.process.kill().expect("Failed to kill Kernel process");
        self.connection_file
            .as_path()
            .to_owned()
            .into_os_string()
            .into_string()
            .expect("Failed to convert connection_file to string");
        std::fs::remove_file(&self.connection_file).expect("Failed to remove connection_file");
    }
}
