// SPDX-License-Identifier: Apache-2.0

#[cfg(unix)]
use libc::{SIGKILL, SIGTERM, kill, setpgid};
use std::{
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};
#[cfg(unix)]
use std::{os::unix::process::CommandExt, time::Duration};

pub(crate) struct RunningProcess {
    child: Child,
    logs: Arc<Mutex<String>>,
    pid_registry: ProcessRegistry,
}

impl RunningProcess {
    pub(crate) fn spawn_dioxus(
        app_port: u16,
        config_path: &std::path::Path,
    ) -> Result<Self, String> {
        let mut command = Command::new("dx");
        command
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
            .stderr(Stdio::piped());

        #[cfg(unix)]
        unsafe {
            command.pre_exec(|| {
                if setpgid(0, 0) == 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::last_os_error())
                }
            });
        }

        let mut child = command
            .spawn()
            .map_err(|error| format!("Could not start dx serve: {error}"))?;

        let logs = Arc::new(Mutex::new(String::new()));
        pipe_child_output(&mut child, Arc::clone(&logs));
        let pid_registry = ProcessRegistry::register(child.id());

        Ok(Self {
            child,
            logs,
            pid_registry,
        })
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
        #[cfg(unix)]
        terminate_process_group(self.child.id());

        let _ = self.child.kill();
        wait_for_exit(&mut self.child);
        self.pid_registry.unregister();
    }
}

pub(crate) fn cleanup_stale_dioxus_processes() {
    for pid in ProcessRegistry::tracked_pids() {
        #[cfg(unix)]
        terminate_process_group(pid);
        ProcessRegistry::unregister_pid(pid);
    }
}

#[cfg(unix)]
fn terminate_process_group(child_id: u32) {
    let pgid = child_id as i32;
    let process_group = -pgid;

    unsafe {
        let _ = kill(process_group, SIGTERM);
    }

    thread::sleep(Duration::from_millis(200));

    unsafe {
        let _ = kill(process_group, SIGKILL);
    }
}

fn wait_for_exit(child: &mut Child) {
    let started = std::time::Instant::now();

    while started.elapsed() < Duration::from_secs(2) {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(_) => return,
        }
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

struct ProcessRegistry {
    pid: u32,
    path: PathBuf,
}

impl ProcessRegistry {
    fn register(pid: u32) -> Self {
        let path = registry_path();
        let mut pids = read_pid_registry(&path);
        if !pids.contains(&pid) {
            pids.push(pid);
            write_pid_registry(&path, &pids);
        }
        Self { pid, path }
    }

    fn tracked_pids() -> Vec<u32> {
        read_pid_registry(&registry_path())
    }

    fn unregister(&self) {
        Self::unregister_pid_from_path(&self.path, self.pid);
    }

    fn unregister_pid(pid: u32) {
        Self::unregister_pid_from_path(&registry_path(), pid);
    }

    fn unregister_pid_from_path(path: &Path, pid: u32) {
        let mut pids = read_pid_registry(path);
        pids.retain(|tracked| *tracked != pid);
        write_pid_registry(path, &pids);
    }
}

fn registry_path() -> PathBuf {
    std::env::temp_dir().join("daneel-e2e-dx-pids.txt")
}

fn read_pid_registry(path: &Path) -> Vec<u32> {
    fs::read_to_string(path)
        .ok()
        .map(|contents| {
            contents
                .lines()
                .filter_map(|line| line.trim().parse::<u32>().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn write_pid_registry(path: &Path, pids: &[u32]) {
    if pids.is_empty() {
        let _ = fs::remove_file(path);
        return;
    }

    let body = pids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(path, format!("{body}\n"));
}
