// SPDX-License-Identifier: Apache-2.0

mod support;

use support::{degraded_app, healthy_app};

#[test]
fn healthy_gateway_dashboard_and_agents_render() {
    let mut app = healthy_app().lock().expect("lock healthy browser test app");

    let dashboard_dom = app.wait_for_dom(
        "/",
        &[
            "Gateway status",
            "Connected to the OpenClaw Gateway over WebSocket (healthy).",
            "Healthy",
        ],
        &["Gateway lookup failed"],
    );
    assert!(dashboard_dom.contains("/assets/main-"));
    assert!(dashboard_dom.contains("Uptime: 123456 ms"));

    let agents_dom = app.wait_for_dom(
        "/agents",
        &[
            "Agent tiles",
            "Active sessions",
            "main",
            "calendar",
            "2/3 active in the last 10 minutes",
        ],
        &["Loading agents", "Gateway lookup failed"],
    );
    assert!(agents_dom.contains("Latest session: agent:main:cron:alpha"));
    assert!(agents_dom.contains("Latest session: agent:calendar:cron:beta"));

    let screenshot = app.capture_screenshot("/agents", "healthy-agents.png");
    app.assert_png(&screenshot, 1440, 1200);
}

#[test]
fn degraded_gateway_dashboard_renders_error_state() {
    let mut app = degraded_app()
        .lock()
        .expect("lock degraded browser test app");

    let dashboard_dom = app.wait_for_dom(
        "/",
        &["Gateway status", "Gateway connection failed", "Degraded"],
        &["Connected to the OpenClaw Gateway over WebSocket (healthy)."],
    );
    assert!(dashboard_dom.contains("Could not open gateway websocket"));

    let screenshot = app.capture_screenshot("/", "degraded-dashboard.png");
    app.assert_png(&screenshot, 1440, 1200);
}
