// SPDX-License-Identifier: Apache-2.0

use std::{
    fs,
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};
use tempfile::{TempDir, tempdir};
use tungstenite::{Message, accept};

const APP_START_TIMEOUT: Duration = Duration::from_secs(180);
const DOM_TIMEOUT: Duration = Duration::from_secs(75);
const VIEWPORT: &str = "1440,1200";
const AUTH_TOKEN: &str = "test-token";

pub const SCREENSHOT_MIN_BYTES: u64 = 20_000;

static TEST_ENVIRONMENT: OnceLock<()> = OnceLock::new();
static HEALTHY_APP: OnceLock<Mutex<BrowserTestApp>> = OnceLock::new();
static DEGRADED_APP: OnceLock<Mutex<BrowserTestApp>> = OnceLock::new();

pub fn healthy_app() -> &'static Mutex<BrowserTestApp> {
    HEALTHY_APP.get_or_init(|| {
        prepare_browser_test_environment();
        Mutex::new(BrowserTestApp::healthy().expect("start healthy browser test app"))
    })
}

pub fn degraded_app() -> &'static Mutex<BrowserTestApp> {
    DEGRADED_APP.get_or_init(|| {
        prepare_browser_test_environment();
        Mutex::new(BrowserTestApp::degraded().expect("start degraded browser test app"))
    })
}

fn prepare_browser_test_environment() {
    TEST_ENVIRONMENT.get_or_init(|| {
        ensure_tool("dx");
        ensure_tool("google-chrome");
        ensure_tool("npm");
        run_command_success(
            Command::new("npm").arg("run").arg("build:css"),
            "build Tailwind CSS for browser integration tests",
        );
    });
}

pub struct BrowserTestApp {
    fixture: TestFixture,
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

        Ok(Self {
            fixture,
            _gateway: gateway,
            process,
            port,
        })
    }

    pub fn wait_for_dom(&mut self, route: &str, required: &[&str], forbidden: &[&str]) -> String {
        dump_dom_until(&self.url(route), required, forbidden)
    }

    pub fn capture_screenshot(&self, route: &str, file_name: &str) -> PathBuf {
        let destination = self.fixture.tempdir.path().join(file_name);
        capture_screenshot(&self.url(route), &destination);
        destination
    }

    pub fn assert_png(&self, path: &Path, expected_width: u32, expected_height: u32) {
        assert_png_dimensions(path, expected_width, expected_height);
        assert!(file_size(path) > SCREENSHOT_MIN_BYTES);
    }

    pub fn url(&self, route: &str) -> String {
        route_url(self.port, route)
    }

    #[allow(dead_code)]
    pub fn logs(&self) -> String {
        self.process.log_output()
    }
}

struct TestFixture {
    tempdir: TempDir,
    config_path: PathBuf,
    gateway_payload: GatewayPayload,
}

impl TestFixture {
    fn healthy() -> Result<Self, String> {
        let tempdir = tempdir().map_err(|error| format!("Could not create tempdir: {error}"))?;
        let config_path = tempdir.path().join("openclaw.json");
        let session_root = tempdir.path().join("sessions");
        fs::create_dir_all(&session_root)
            .map_err(|error| format!("Could not create session root: {error}"))?;

        let now_ms = now_ms();
        let main_path = session_root.join("main.json");
        let calendar_path = session_root.join("calendar.json");
        let planner_path = session_root.join("planner.json");

        write_session_store(
            &main_path,
            &[now_ms - 120_000, now_ms - 240_000, now_ms - 7_200_000],
        )?;
        write_session_store(&calendar_path, &[now_ms - 300_000, now_ms - 8_400_000])?;
        write_session_store(&planner_path, &[now_ms - 8_400_000])?;

        Ok(Self {
            tempdir,
            config_path,
            gateway_payload: GatewayPayload {
                state_version: 42,
                uptime_ms: 123_456,
                snapshot_ts: now_ms,
                agents: vec![
                    MockAgent::new(
                        "main",
                        true,
                        true,
                        &main_path,
                        "agent:main:cron:alpha",
                        120_000,
                    ),
                    MockAgent::new(
                        "calendar",
                        false,
                        true,
                        &calendar_path,
                        "agent:calendar:cron:beta",
                        300_000,
                    ),
                    MockAgent::new(
                        "planner",
                        false,
                        false,
                        &planner_path,
                        "agent:planner:cron:gamma",
                        8_400_000,
                    ),
                ],
            },
        })
    }

    fn degraded() -> Result<Self, String> {
        let tempdir = tempdir().map_err(|error| format!("Could not create tempdir: {error}"))?;
        let config_path = tempdir.path().join("openclaw.json");
        Ok(Self {
            tempdir,
            config_path,
            gateway_payload: GatewayPayload {
                state_version: 0,
                uptime_ms: 0,
                snapshot_ts: now_ms(),
                agents: vec![],
            },
        })
    }

    fn write_openclaw_config(&self, gateway_addr: SocketAddr) -> Result<(), String> {
        let payload = json!({
            "gateway": {
                "port": gateway_addr.port(),
                "auth": {
                    "token": AUTH_TOKEN
                }
            }
        });
        fs::write(
            &self.config_path,
            serde_json::to_vec_pretty(&payload).unwrap(),
        )
        .map_err(|error| format!("Could not write test OpenClaw config: {error}"))
    }
}

#[derive(Clone)]
struct GatewayPayload {
    state_version: u64,
    uptime_ms: u64,
    snapshot_ts: u64,
    agents: Vec<MockAgent>,
}

#[derive(Clone)]
struct MockAgent {
    id: String,
    is_default: bool,
    heartbeat_enabled: bool,
    session_store_path: PathBuf,
    latest_session_key: String,
    latest_activity_age_ms: u64,
}

impl MockAgent {
    fn new(
        id: &str,
        is_default: bool,
        heartbeat_enabled: bool,
        session_store_path: &Path,
        latest_session_key: &str,
        latest_activity_age_ms: u64,
    ) -> Self {
        Self {
            id: id.to_string(),
            is_default,
            heartbeat_enabled,
            session_store_path: session_store_path.to_path_buf(),
            latest_session_key: latest_session_key.to_string(),
            latest_activity_age_ms,
        }
    }

    fn to_value(&self) -> Value {
        json!({
            "agentId": self.id,
            "name": self.id,
            "isDefault": self.is_default,
            "heartbeat": {
                "enabled": self.heartbeat_enabled,
                "every": if self.heartbeat_enabled { "120m" } else { "disabled" },
                "model": if self.is_default {
                    Value::String("default".to_string())
                } else {
                    Value::Null
                }
            },
            "sessions": {
                "path": self.session_store_path.display().to_string(),
                "recent": [
                    {
                        "key": self.latest_session_key,
                        "age": self.latest_activity_age_ms
                    }
                ]
            }
        })
    }
}

struct MockGateway {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl MockGateway {
    fn spawn(payload: GatewayPayload) -> Result<Self, String> {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .map_err(|error| format!("bind mock gateway: {error}"))?;
        let addr = listener
            .local_addr()
            .map_err(|error| format!("mock gateway local_addr: {error}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| format!("mock gateway set_nonblocking: {error}"))?;

        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if let Err(error) = handle_gateway_client(stream, &payload) {
                            eprintln!("mock gateway client error: {error}");
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(25));
                    }
                    Err(error) => {
                        eprintln!("mock gateway accept error: {error}");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            addr,
            stop,
            thread: Some(handle),
        })
    }

    fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Drop for MockGateway {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

fn handle_gateway_client(stream: TcpStream, payload: &GatewayPayload) -> Result<(), String> {
    let mut socket =
        accept(stream).map_err(|error| format!("websocket handshake failed: {error}"))?;

    loop {
        let message = match socket.read() {
            Ok(message) => message,
            Err(tungstenite::Error::ConnectionClosed) | Err(tungstenite::Error::AlreadyClosed) => {
                return Ok(());
            }
            Err(error) => return Err(format!("read gateway message: {error}")),
        };

        let Message::Text(text) = message else {
            continue;
        };
        let request: Value =
            serde_json::from_str(&text).map_err(|error| format!("parse request json: {error}"))?;

        let response = match request.get("method").and_then(Value::as_str) {
            Some("connect") => connect_response(&request, payload),
            Some("health") => health_response(&request, payload),
            Some(method) => error_response(&request, format!("Unsupported method {method}")),
            None => error_response(&request, "Missing method".to_string()),
        };

        socket
            .send(Message::Text(response.to_string().into()))
            .map_err(|error| format!("send gateway response: {error}"))?;
    }
}

fn connect_response(request: &Value, payload: &GatewayPayload) -> Value {
    let id = request
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("connect");
    json!({
        "type": "res",
        "id": id,
        "ok": true,
        "payload": {
            "protocolVersion": 3,
            "stateVersion": payload.state_version,
            "uptimeMs": payload.uptime_ms,
            "snapshot": {
                "health": {
                    "ts": payload.snapshot_ts,
                    "defaultAgentId": "main",
                    "agents": payload.agents.iter().map(MockAgent::to_value).collect::<Vec<_>>()
                }
            }
        }
    })
}

fn health_response(request: &Value, payload: &GatewayPayload) -> Value {
    let id = request
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("health");
    json!({
        "type": "res",
        "id": id,
        "ok": true,
        "payload": {
            "status": "healthy",
            "stateVersion": payload.state_version,
            "uptimeMs": payload.uptime_ms
        }
    })
}

fn error_response(request: &Value, message: String) -> Value {
    let id = request.get("id").and_then(Value::as_str).unwrap_or("error");
    json!({
        "type": "res",
        "id": id,
        "ok": false,
        "error": {
            "message": message
        }
    })
}

struct RunningProcess {
    child: Child,
    logs: Arc<Mutex<String>>,
}

impl RunningProcess {
    fn spawn_dioxus(app_port: u16, config_path: &Path) -> Result<Self, String> {
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

    fn assert_still_running(&mut self) {
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

    fn log_output(&self) -> String {
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
    let mut buffer = String::new();
    let _ = pipe.read_to_string(&mut buffer);
    logs.lock()
        .expect("lock logs for pipe write")
        .push_str(&buffer);
}

fn wait_for_http_ready(port: u16, app: &mut RunningProcess) {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let started = Instant::now();

    while started.elapsed() < APP_START_TIMEOUT {
        app.assert_still_running();
        if TcpStream::connect_timeout(&address, Duration::from_millis(250)).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(500));
    }

    panic!(
        "Timed out waiting for http://127.0.0.1:{port}.\nCaptured logs:\n{}",
        app.log_output()
    );
}

fn dump_dom_until(url: &str, required: &[&str], forbidden: &[&str]) -> String {
    let started = Instant::now();
    let mut last_dom = String::new();

    while started.elapsed() < DOM_TIMEOUT {
        let output = run_command_capture(
            Command::new("google-chrome")
                .arg("--headless=new")
                .arg("--disable-gpu")
                .arg("--no-sandbox")
                .arg("--virtual-time-budget=20000")
                .arg("--dump-dom")
                .arg(url),
            "capture browser DOM",
        );
        let dom = String::from_utf8_lossy(&output.stdout).into_owned();
        last_dom = dom.clone();

        let has_required = required.iter().all(|text| dom.contains(text));
        let has_forbidden = forbidden.iter().any(|text| dom.contains(text));
        if has_required && !has_forbidden {
            return dom;
        }

        thread::sleep(Duration::from_millis(750));
    }

    panic!(
        "Timed out waiting for browser DOM at {url}.\nRequired: {required:?}\nForbidden: {forbidden:?}\nLast DOM:\n{last_dom}"
    );
}

fn capture_screenshot(url: &str, destination: &Path) {
    run_command_success(
        Command::new("google-chrome")
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--virtual-time-budget=20000")
            .arg(format!("--window-size={VIEWPORT}"))
            .arg(format!("--screenshot={}", destination.display()))
            .arg(url),
        "capture browser screenshot",
    );
}

fn assert_png_dimensions(path: &Path, expected_width: u32, expected_height: u32) {
    let bytes = fs::read(path).expect("read PNG screenshot");
    assert!(bytes.len() > 24, "PNG screenshot is too small");
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    let width = u32::from_be_bytes(bytes[16..20].try_into().expect("PNG width bytes"));
    let height = u32::from_be_bytes(bytes[20..24].try_into().expect("PNG height bytes"));
    assert_eq!(width, expected_width, "unexpected screenshot width");
    assert_eq!(height, expected_height, "unexpected screenshot height");
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path).expect("read screenshot metadata").len()
}

fn run_command_success(command: &mut Command, context: &str) {
    let output = run_command_capture(command, context);
    if !output.status.success() {
        panic!(
            "{context} failed with status {}.\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}

fn run_command_capture(command: &mut Command, context: &str) -> Output {
    command.output().unwrap_or_else(|error| {
        panic!("{context} failed to start: {error}");
    })
}

fn ensure_tool(tool: &str) {
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

fn pick_unused_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("bind an ephemeral port")
        .local_addr()
        .expect("read ephemeral port")
        .port()
}

fn route_url(port: u16, route: &str) -> String {
    format!("http://127.0.0.1:{port}{route}")
}

fn write_session_store(path: &Path, updated_times: &[u64]) -> Result<(), String> {
    let payload = updated_times
        .iter()
        .enumerate()
        .map(|(index, updated_at)| {
            (
                format!("session-{index}"),
                json!({
                    "updatedAt": updated_at
                }),
            )
        })
        .collect::<serde_json::Map<String, Value>>();

    fs::write(
        path,
        serde_json::to_vec_pretty(&Value::Object(payload)).unwrap(),
    )
    .map_err(|error| format!("Could not write session store {}: {error}", path.display()))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis() as u64
}
