use super::process_set_up::EXE_PATH;
use super::testcase::TestCase;

use std::fs::OpenOptions;
use std::process::{Command, Stdio};
use std::time::Duration;

impl TestCase {
    pub fn start_server(&mut self, restart: bool) {
        let mut open_option = OpenOptions::new();
        open_option.create(true);
        if restart {
            open_option.append(false);
        } else {
            open_option.write(true).truncate(true);
        }

        let stdout = open_option
            .open(&self.stdout_file)
            .expect("failed to create stdout file");
        let stderr = open_option
            .open(&self.stderr_file)
            .expect("failed to create stderr file");

        let command = Command::new(EXE_PATH.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .expect(
                format!(
                    "case {} incorrect: failed to execute server process",
                    self.name
                )
                .as_str(),
            );
        self.running_process = Some(command);
        // sleep 1 second for server startup
        std::thread::sleep(Duration::from_secs(1));
    }

    pub fn kill_server(&mut self) {
        if let Some(mut child) = self.running_process.take() {
            child.kill().expect(
                format!("case {} incorrect: cannot kill server process", self.name).as_str(),
            );
        }
    }
}
