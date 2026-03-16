// SPDX-License-Identifier: Apache-2.0

use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

pub(crate) struct RunningProcess {
    child: Child,
    logs: Arc<Mutex<String>>,
}

impl RunningProcess {
    pub(crate) fn spawn_dioxus(
        app_port: u16,
        config_path: &std::path::Path,
    ) -> Result<Self, String> {
        let mut child = Command::new("dx")
            .arg("serve")
            .arg("--web")
            .arg("--fullstack")
            .arg("--addr")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(app_port.to_string())
            .arg("--open")
            .arg("false")
            .env("OPENCLAW_CONFIG_PATH", config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("Could not start dx serve: {error}"))?;

        let logs = Arc::new(Mutex::new(String::new()));
        pipe_child_output(&mut child, Arc::clone(&logs));

        Ok(Self { child, logs })
    }

    pub(crate) fn assert_still_running(&mut self) {
        if let Some(status) = self
            .child
            .try_wait()
            .expect("check if Dioxus app is still running")
        {
            panic!(
                "Dioxus app exited unexpectedly with {status}.\nCaptured logs:\n{}",
                self.log_output()
            );
        }
    }

    pub(crate) fn log_output(&self) -> String {
        self.logs.lock().expect("lock app logs").clone()
    }
}

impl Drop for RunningProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn pipe_child_output(child: &mut Child, logs: Arc<Mutex<String>>) {
    if let Some(stdout) = child.stdout.take() {
        let logs = Arc::clone(&logs);
        thread::spawn(move || read_pipe(stdout, logs));
    }

    if let Some(stderr) = child.stderr.take() {
        thread::spawn(move || read_pipe(stderr, logs));
    }
}

fn read_pipe<T: Read>(mut pipe: T, logs: Arc<Mutex<String>>) {
    let mut reader = BufReader::new(&mut pipe);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => logs
                .lock()
                .expect("lock logs for pipe write")
                .push_str(&line),
            Err(_) => break,
        }
    }
}
