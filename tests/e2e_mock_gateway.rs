// SPDX-License-Identifier: Apache-2.0

mod support;

use serial_test::serial;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use support::{with_degraded_app, with_empty_graph_app, with_healthy_app, with_smoke_app};

const PAGE_OK: &str = "HTTP/1.1 200 OK";
const PAGE_ERROR: &str = "Internal Server Error";
const DASHBOARD_REQUIRED: &[&str] = &[PAGE_OK, "Mission Control", "Gateway status"];
const AGENTS_REQUIRED: &[&str] = &[PAGE_OK, "Graph View", "Loading agents"];

fn expect_dashboard_shell(app: &mut support::BrowserTestApp) -> String {
    let response = app.wait_for_page_response("/", DASHBOARD_REQUIRED, &[PAGE_ERROR]);
    assert!(response.contains("/assets/main-"));
    response
}

fn expect_agents_shell(app: &mut support::BrowserTestApp) -> String {
    let response = app.wait_for_page_response("/agents", AGENTS_REQUIRED, &[PAGE_ERROR]);
    assert!(response.contains("/assets/main-"));
    response
}

fn temp_artifact_path(name: &str, extension: &str) -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir()
        .join(format!("daneel-{name}-{stamp}.{extension}"))
        .display()
        .to_string()
}

#[test]
#[serial]
fn healthy_gateway_event_stream_replays_latest_health_state() {
    with_healthy_app(|app| {
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
    });
}

#[test]
#[serial]
fn healthy_gateway_dashboard_and_agents_render() {
    with_healthy_app(|app| {
        let _ = expect_dashboard_shell(app);
        let _ = expect_agents_shell(app);
    });
}

#[test]
#[serial]
fn degraded_gateway_event_stream_replays_reconnecting_state() {
    with_degraded_app(|app| {
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
    });
}

#[test]
#[serial]
fn degraded_gateway_dashboard_renders_error_state() {
    with_degraded_app(|app| {
        let _ = expect_dashboard_shell(app);
    });
}

#[test]
#[serial]
fn empty_graph_gateway_dashboard_renders_empty_state_without_errors() {
    with_empty_graph_app(|app| {
        let dashboard = expect_dashboard_shell(app);

        assert!(dashboard.contains("Gateway status"));
        assert!(dashboard.contains("Agents graph"));
        assert!(dashboard.contains("/assets/main-"));
        assert!(!dashboard.contains("Internal Server Error"));
    });
}

#[test]
#[serial]
fn agents_view_renders_loading_shell_without_errors() {
    with_healthy_app(|app| {
        let _ = expect_agents_shell(app);
    });
}

#[test]
#[serial]
fn dashboard_route_remains_available_across_repeated_requests() {
    with_healthy_app(|app| {
        let _ = expect_dashboard_shell(app);
        let _ = expect_dashboard_shell(app);
    });
}

#[test]
#[serial]
fn agents_route_remains_available_across_repeated_requests() {
    with_healthy_app(|app| {
        let _ = expect_agents_shell(app);
        let _ = expect_agents_shell(app);
    });
}

#[test]
#[serial]
fn healthy_gateway_polished_routes_pass_browser_capture_verification() {
    with_healthy_app(|app| {
        let dashboard_png = temp_artifact_path("dashboard-polish", "png");
        let dashboard_dom = temp_artifact_path("dashboard-polish", "html");
        let agents_png = temp_artifact_path("agents-polish", "png");
        let agents_dom = temp_artifact_path("agents-polish", "html");

        app.verify_route_capture(
            "/",
            &dashboard_png,
            &dashboard_dom,
            &["Mission Control", "Gateway status", "Agents graph"],
            &[PAGE_ERROR],
        );
        app.verify_route_capture_with_expectations(
            "/agents",
            &agents_png,
            &agents_dom,
            &["Graph View"],
            &["Gateway lookup failed"],
            &[
                "[data-sidebar-polish=\"enhanced\"]",
                "[data-agents-route=\"enhanced\"]",
            ],
            &[],
            std::time::Duration::from_secs(60),
        );

        let dashboard_html = fs::read_to_string(&dashboard_dom).expect("read dashboard DOM");
        let agents_html = fs::read_to_string(&agents_dom).expect("read agents DOM");

        assert!(dashboard_html.contains("data-visual-shell=\"mission-control\""));
        assert!(dashboard_html.contains("data-polish-hero=\"dashboard\""));
        assert!(dashboard_html.contains("data-dashboard-panel=\"graph\""));
        assert!(dashboard_html.contains("data-summary-polish=\"enhanced\""));
        assert!(agents_html.contains("data-sidebar-polish=\"enhanced\""));
        assert!(agents_html.contains("data-agents-route=\"enhanced\""));

        let _ = fs::remove_file(&dashboard_png);
        let _ = fs::remove_file(&dashboard_dom);
        let _ = fs::remove_file(&agents_png);
        let _ = fs::remove_file(&agents_dom);
    });
}

#[test]
#[serial]
fn degraded_gateway_polished_dashboard_keeps_shell_and_graph_markers() {
    with_degraded_app(|app| {
        let screenshot = temp_artifact_path("dashboard-degraded-polish", "png");
        let dom = temp_artifact_path("dashboard-degraded-polish", "html");

        app.verify_route_capture(
            "/",
            &screenshot,
            &dom,
            &["Mission Control", "Gateway status"],
            &[PAGE_ERROR],
        );

        let dashboard_html = fs::read_to_string(&dom).expect("read degraded dashboard DOM");
        assert!(dashboard_html.contains("data-visual-shell=\"mission-control\""));
        assert!(dashboard_html.contains("data-dashboard-panel=\"gateway\""));
        assert!(dashboard_html.contains("data-graph-polish=\"enhanced\""));

        let _ = fs::remove_file(&screenshot);
        let _ = fs::remove_file(&dom);
    });
}

#[test]
#[serial]
fn poc_smoke_dashboard_shell_and_graph_snapshot_cover_full_vertical_slice() {
    with_smoke_app(|app| {
        let event_stream = app.wait_for_gateway_event(
            &[
                PAGE_OK,
                "content-type: text/event-stream",
                "\"level\":\"healthy\"",
                "\"summary\":\"Gateway health update: healthy.\"",
            ],
            &[],
        );
        assert!(event_stream.contains("\"detail\":\"Live gateway event received.\""));

        let screenshot = temp_artifact_path("dashboard-smoke", "png");
        let dom = temp_artifact_path("dashboard-smoke", "html");

        app.verify_route_capture_with_expectations(
            "/",
            &screenshot,
            &dom,
            &["Mission Control", "Gateway status", "Agents graph"],
            &[PAGE_ERROR],
            &[
                "[data-visual-shell=\"mission-control\"]",
                "[data-polish-hero=\"dashboard\"]",
                "[data-dashboard-panel=\"graph\"]",
                "[data-graph-polish=\"enhanced\"]",
            ],
            &[],
            std::time::Duration::from_secs(60),
        );

        let dashboard_html = fs::read_to_string(&dom).expect("read smoke dashboard DOM");

        assert!(dashboard_html.contains("data-visual-shell=\"mission-control\""));
        assert!(dashboard_html.contains("data-polish-hero=\"dashboard\""));
        assert!(dashboard_html.contains("data-dashboard-panel=\"graph\""));
        assert!(dashboard_html.contains("data-graph-polish=\"enhanced\""));

        let _ = fs::remove_file(&screenshot);
        let _ = fs::remove_file(&dom);
    });
}
