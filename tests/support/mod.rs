// SPDX-License-Identifier: Apache-2.0

mod app;
mod fixture;
mod gateway;
mod process;
mod util;

use std::sync::{Mutex, OnceLock};

pub use app::BrowserTestApp;
use util::{COMMAND_TIMEOUT, ensure_tool, run_command_success};

static TEST_ENVIRONMENT: OnceLock<()> = OnceLock::new();
static HEALTHY_APP: OnceLock<Mutex<BrowserTestApp>> = OnceLock::new();
static DEGRADED_APP: OnceLock<Mutex<BrowserTestApp>> = OnceLock::new();

pub fn healthy_app() -> &'static Mutex<BrowserTestApp> {
    init_browser_test_app(
        &HEALTHY_APP,
        BrowserTestApp::healthy,
        "start healthy browser test app",
    )
}

pub fn degraded_app() -> &'static Mutex<BrowserTestApp> {
    init_browser_test_app(
        &DEGRADED_APP,
        BrowserTestApp::degraded,
        "start degraded browser test app",
    )
}

fn init_browser_test_app(
    slot: &'static OnceLock<Mutex<BrowserTestApp>>,
    builder: fn() -> Result<BrowserTestApp, String>,
    context: &str,
) -> &'static Mutex<BrowserTestApp> {
    slot.get_or_init(|| {
        prepare_browser_test_environment();
        Mutex::new(builder().unwrap_or_else(|error| {
            panic!("{context} failed: {error}");
        }))
    })
}

fn prepare_browser_test_environment() {
    TEST_ENVIRONMENT.get_or_init(|| {
        ensure_tool("dx");
        ensure_tool("npm");
        run_command_success(
            std::process::Command::new("npm")
                .arg("run")
                .arg("build:css"),
            "build Tailwind CSS for browser integration tests",
            COMMAND_TIMEOUT,
        );
    });
}
