// SPDX-License-Identifier: Apache-2.0

mod app;
mod command;
mod data;
mod fixture;
mod gateway;
mod http;
mod process;

use std::sync::{Mutex, OnceLock};

pub use app::BrowserTestApp;
use command::{COMMAND_TIMEOUT, ensure_tool, run_command_success};
use process::cleanup_stale_dioxus_processes;

static TEST_ENVIRONMENT: OnceLock<()> = OnceLock::new();
static CLEANUP: OnceLock<()> = OnceLock::new();
static HEALTHY_APP: OnceLock<Mutex<Option<BrowserTestApp>>> = OnceLock::new();
static EMPTY_GRAPH_APP: OnceLock<Mutex<Option<BrowserTestApp>>> = OnceLock::new();
static DEGRADED_APP: OnceLock<Mutex<Option<BrowserTestApp>>> = OnceLock::new();
static SMOKE_APP: OnceLock<Mutex<Option<BrowserTestApp>>> = OnceLock::new();

pub fn with_healthy_app<T>(f: impl FnOnce(&mut BrowserTestApp) -> T) -> T {
    with_browser_test_app(
        &HEALTHY_APP,
        BrowserTestApp::healthy,
        "start healthy browser test app",
        f,
    )
}

pub fn with_degraded_app<T>(f: impl FnOnce(&mut BrowserTestApp) -> T) -> T {
    with_browser_test_app(
        &DEGRADED_APP,
        BrowserTestApp::degraded,
        "start degraded browser test app",
        f,
    )
}

pub fn with_empty_graph_app<T>(f: impl FnOnce(&mut BrowserTestApp) -> T) -> T {
    with_browser_test_app(
        &EMPTY_GRAPH_APP,
        BrowserTestApp::empty_graph,
        "start empty-graph browser test app",
        f,
    )
}

pub fn with_smoke_app<T>(f: impl FnOnce(&mut BrowserTestApp) -> T) -> T {
    with_browser_test_app(
        &SMOKE_APP,
        BrowserTestApp::smoke,
        "start smoke browser test app",
        f,
    )
}

fn with_browser_test_app<T>(
    slot: &'static OnceLock<Mutex<Option<BrowserTestApp>>>,
    builder: fn() -> Result<BrowserTestApp, String>,
    context: &str,
    f: impl FnOnce(&mut BrowserTestApp) -> T,
) -> T {
    let app = slot.get_or_init(|| {
        prepare_browser_test_environment();
        register_browser_test_cleanup();
        Mutex::new(Some(builder().unwrap_or_else(|error| {
            panic!("{context} failed: {error}");
        })))
    });

    let mut guard = match app.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("recovering poisoned browser test app state after an earlier test failure");
            poisoned.into_inner()
        }
    };

    let app = guard
        .as_mut()
        .unwrap_or_else(|| panic!("{context} was already cleaned up"));
    f(app)
}

fn prepare_browser_test_environment() {
    TEST_ENVIRONMENT.get_or_init(|| {
        ensure_tool("dx");
        ensure_tool("npm");
        cleanup_stale_dioxus_processes();
        run_command_success(
            std::process::Command::new("npm")
                .arg("run")
                .arg("build:css"),
            "build Tailwind CSS for browser integration tests",
            COMMAND_TIMEOUT,
        );
    });
}

fn register_browser_test_cleanup() {
    CLEANUP.get_or_init(|| unsafe {
        libc::atexit(cleanup_browser_test_apps);
    });
}

extern "C" fn cleanup_browser_test_apps() {
    cleanup_browser_test_slot(&HEALTHY_APP);
    cleanup_browser_test_slot(&EMPTY_GRAPH_APP);
    cleanup_browser_test_slot(&DEGRADED_APP);
    cleanup_browser_test_slot(&SMOKE_APP);
}

fn cleanup_browser_test_slot(slot: &'static OnceLock<Mutex<Option<BrowserTestApp>>>) {
    if let Some(app) = slot.get() {
        match app.lock() {
            Ok(mut guard) => {
                *guard = None;
            }
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                *guard = None;
            }
        }
    }
}
