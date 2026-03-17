// SPDX-License-Identifier: Apache-2.0

mod support;

use serial_test::serial;
use std::sync::{Mutex, MutexGuard};

use support::{BrowserTestApp, degraded_app, healthy_app};

const PAGE_OK: &str = "HTTP/1.1 200 OK";
const PAGE_ERROR: &str = "Internal Server Error";
const DASHBOARD_REQUIRED: &[&str] = &[PAGE_OK, "Mission Control", "Gateway status"];
const AGENTS_REQUIRED: &[&str] = &[PAGE_OK, "Graph View", "Loading agents"];

fn lock_app<'a>(app: &'a Mutex<BrowserTestApp>, label: &str) -> MutexGuard<'a, BrowserTestApp> {
    match app.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("recovering poisoned {label} app state after an earlier test failure");
            poisoned.into_inner()
        }
    }
}

fn expect_dashboard_shell(app: &mut BrowserTestApp) -> String {
    let response = app.wait_for_page_response("/", DASHBOARD_REQUIRED, &[PAGE_ERROR]);
    assert!(response.contains("/assets/main-"));
    response
}

fn expect_agents_shell(app: &mut BrowserTestApp) -> String {
    let response = app.wait_for_page_response("/agents", AGENTS_REQUIRED, &[PAGE_ERROR]);
    assert!(response.contains("/assets/main-"));
    response
}

#[test]
#[serial]
fn healthy_gateway_event_stream_replays_latest_health_state() {
    let mut app = lock_app(healthy_app(), "healthy");

    let event_stream = app.wait_for_gateway_event(
        &[
            "HTTP/1.1 200 OK",
            "content-type: text/event-stream",
            "\"level\":\"healthy\"",
            "\"summary\":\"Gateway health update: healthy.\"",
        ],
        &[],
    );

    assert!(event_stream.contains("\"detail\":\"Live gateway event received.\""));
}

#[test]
#[serial]
fn healthy_gateway_dashboard_and_agents_render() {
    let mut app = lock_app(healthy_app(), "healthy");

    let _ = expect_dashboard_shell(&mut app);
    let _ = expect_agents_shell(&mut app);
}

#[test]
#[serial]
fn degraded_gateway_event_stream_replays_reconnecting_state() {
    let mut app = lock_app(degraded_app(), "degraded");

    let event_stream = app.wait_for_gateway_event(
        &[
            "HTTP/1.1 200 OK",
            "content-type: text/event-stream",
            "\"level\":\"connecting\"",
            "Gateway event stream reconnecting",
        ],
        &[],
    );

    assert!(event_stream.contains("Could not open gateway websocket"));
}

#[test]
#[serial]
fn degraded_gateway_dashboard_renders_error_state() {
    let mut app = lock_app(degraded_app(), "degraded");

    let _ = expect_dashboard_shell(&mut app);
}

#[test]
#[serial]
fn agents_view_renders_loading_shell_without_errors() {
    let mut app = lock_app(healthy_app(), "healthy");

    let _ = expect_agents_shell(&mut app);
}

#[test]
#[serial]
fn dashboard_route_remains_available_across_repeated_requests() {
    let mut app = lock_app(healthy_app(), "healthy");

    let _ = expect_dashboard_shell(&mut app);
    let _ = expect_dashboard_shell(&mut app);
}

#[test]
#[serial]
fn agents_route_remains_available_across_repeated_requests() {
    let mut app = lock_app(healthy_app(), "healthy");

    let _ = expect_agents_shell(&mut app);
    let _ = expect_agents_shell(&mut app);
}
