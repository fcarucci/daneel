// SPDX-License-Identifier: Apache-2.0

use super::{
    fixture::TestFixture,
    gateway::MockGateway,
    process::RunningProcess,
    util::{
        pick_unused_port, read_http_until, read_sse_until, wait_for_backend_route_ready,
        wait_for_http_ready, with_query_param,
    },
};
use std::net::SocketAddr;

pub struct BrowserTestApp {
    _fixture: TestFixture,
    _gateway: Option<MockGateway>,
    process: RunningProcess,
    port: u16,
}

impl BrowserTestApp {
    pub fn healthy() -> Result<Self, String> {
        let fixture = TestFixture::healthy()?;
        let gateway = MockGateway::spawn(fixture.gateway_payload.clone())?;
        fixture.write_openclaw_config(gateway.addr())?;
        Self::start(fixture, Some(gateway))
    }

    pub fn degraded() -> Result<Self, String> {
        let fixture = TestFixture::degraded()?;
        fixture.write_openclaw_config(SocketAddr::from(([127, 0, 0, 1], pick_unused_port())))?;
        Self::start(fixture, None)
    }

    fn start(fixture: TestFixture, gateway: Option<MockGateway>) -> Result<Self, String> {
        let port = pick_unused_port();
        let mut process = RunningProcess::spawn_dioxus(port, &fixture.config_path)?;
        wait_for_http_ready(port, &mut process);
        wait_for_backend_route_ready(port, "/api/gateway/events", &mut process);

        Ok(Self {
            _fixture: fixture,
            _gateway: gateway,
            process,
            port,
        })
    }

    pub fn wait_for_page_response(
        &mut self,
        route: &str,
        required: &[&str],
        forbidden: &[&str],
    ) -> String {
        self.process.assert_still_running();
        read_http_until(
            self.port,
            &with_query_param(route, "e2e-disable-live", "1"),
            "text/html",
            required,
            forbidden,
            &mut self.process,
        )
    }

    pub fn wait_for_gateway_event(&mut self, required: &[&str], forbidden: &[&str]) -> String {
        self.process.assert_still_running();
        read_sse_until(
            self.port,
            "/api/gateway/events",
            required,
            forbidden,
            &mut self.process,
        )
    }

    #[allow(dead_code)]
    pub fn logs(&self) -> String {
        self.process.log_output()
    }
}
