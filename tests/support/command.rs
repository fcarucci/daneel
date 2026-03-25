// SPDX-License-Identifier: Apache-2.0

use std::{
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

pub const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

pub fn ensure_tool(tool: &str) {
    let output = Command::new("bash")
        .arg("-lc")
        .arg(format!("command -v {tool}"))
        .output()
        .unwrap_or_else(|error| panic!("Could not check for {tool}: {error}"));

    assert!(
        output.status.success(),
        "Required tool `{tool}` was not found on PATH"
    );
}

pub fn run_command_success(command: &mut Command, context: &str, timeout: Duration) {
    let output = run_command_capture(command, context, timeout);
    if !output.status.success() {
        panic!(
            "{context} failed with status {}.\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}

pub fn run_command_capture(command: &mut Command, context: &str, timeout: Duration) -> Output {
    let command_line = format!("{command:?}");
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().unwrap_or_else(|error| {
        panic!("{context} failed to start ({command_line}): {error}");
    });
    let started = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .unwrap_or_else(|error| panic!("{context} failed to collect output: {error}"));
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let output = child.wait_with_output().unwrap_or_else(|error| {
                    panic!("{context} timed out and failed to collect output: {error}");
                });
                panic!(
                    "{context} timed out after {}s for {command_line}.\nstdout:\n{}\nstderr:\n{}",
                    timeout.as_secs(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                );
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(error) => panic!("{context} failed while waiting for {command_line}: {error}"),
        }
    }
}
