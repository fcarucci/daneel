// SPDX-License-Identifier: Apache-2.0

mod support;

use serial_test::serial;
use std::sync::{Mutex, MutexGuard};

use support::{BrowserTestApp, degraded_app, healthy_app};

fn lock_app<'a>(app: &'a Mutex<BrowserTestApp>, label: &str) -> MutexGuard<'a, BrowserTestApp> {
    match app.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("recovering poisoned {label} app state after an earlier test failure");
            poisoned.into_inner()
        }
    }
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

    let dashboard_response = app.wait_for_page_response(
        "/",
        &["HTTP/1.1 200 OK", "Mission Control", "Gateway status"],
        &["Internal Server Error"],
    );
    assert!(dashboard_response.contains("/assets/main-"));

    let agents_response = app.wait_for_page_response(
        "/agents",
        &["HTTP/1.1 200 OK", "Graph View", "Loading agents"],
        &["Internal Server Error"],
    );
    assert!(agents_response.contains("/assets/main-"));
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

    let dashboard_response = app.wait_for_page_response(
        "/",
        &["HTTP/1.1 200 OK", "Mission Control", "Gateway status"],
        &["Internal Server Error"],
    );
    assert!(dashboard_response.contains("/assets/main-"));
}

#[test]
#[serial]
fn agents_view_renders_time_ago_ribbons() {
    let mut app = lock_app(healthy_app(), "healthy");

    let agents_response = app.wait_for_page_response(
        "/agents",
        &["HTTP/1.1 200 OK", "Graph View", "Agent tiles"],
        &["Internal Server Error"],
    );
    assert!(agents_response.contains("120s ago"));
    assert!(agents_response.contains("5m ago"));
    assert!(agents_response.contains("2h 20m ago"));
}

#[test]
#[serial]
fn inactive_agent_tile_has_no_active_glow() {
    let mut app = lock_app(healthy_app(), "healthy");

    let agents_response = app.wait_for_page_response(
        "/agents",
        &["HTTP/1.1 200 OK", "Graph View", "Agent tiles"],
        &["Internal Server Error"],
    );
    // Planner agent should be inactive (8.4M ms old)
    assert!(agents_response.contains("planner"));
    assert!(!agents_response.contains("border-emerald-300/35")); // Active border class
    assert!(agents_response.contains("border-white/10")); // Inactive border class
}

#[test]
#[serial]
fn disabled_heartbeat_agent_renders_gray_heart() {
    let mut app = lock_app(healthy_app(), "healthy");

    let agents_response = app.wait_for_page_response(
        "/agents",
        &["HTTP/1.1 200 OK", "Graph View", "Agent tiles"],
        &["Internal Server Error"],
    );
    assert!(agents_response.contains("planner"));
    assert!(agents_response.contains("Heartbeat disabled"));
    assert!(agents_response.contains("text-slate-500"));
    assert!(!agents_response.contains("text-rose-400"));
}
