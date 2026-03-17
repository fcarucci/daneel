// SPDX-License-Identifier: Apache-2.0

use std::{env, fs, path::PathBuf};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct OpenClawConfig {
    #[serde(default)]
    gateway: GatewayConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct GatewayConfig {
    #[serde(default = "default_gateway_port")]
    port: u16,
    #[serde(default)]
    auth: OpenClawAuth,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct OpenClawAuth {
    #[serde(default)]
    token: String,
}

#[derive(Clone, Debug)]
pub(crate) struct LoadedGatewayConfig {
    pub(crate) token: String,
    pub(crate) ws_url: String,
}

impl LoadedGatewayConfig {
    pub(crate) fn new(token: impl Into<String>, ws_url: impl Into<String>) -> Result<Self, String> {
        let token = token.into();
        if token.trim().is_empty() {
            return Err("No gateway auth token was found in the OpenClaw config.".to_string());
        }

        let ws_url = ws_url.into();
        if ws_url.trim().is_empty() {
            return Err("No gateway websocket URL was configured.".to_string());
        }

        Ok(Self { token, ws_url })
    }
}

pub(crate) const DEFAULT_GATEWAY_URL: &str = "ws://127.0.0.1:18789";

pub(crate) fn load_gateway_config() -> Result<LoadedGatewayConfig, String> {
    let path = openclaw_config_path()?;
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let parsed: OpenClawConfig = serde_json::from_str(&raw)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))?;

    LoadedGatewayConfig::new(
        parsed.gateway.auth.token,
        format!("ws://127.0.0.1:{}/", parsed.gateway.port),
    )
    .map_err(|error| {
        if error.contains("No gateway auth token") {
            format!("No gateway auth token was found in {}.", path.display())
        } else {
            error
        }
    })
}

fn openclaw_config_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("OPENCLAW_CONFIG_PATH") {
        return Ok(PathBuf::from(path));
    }

    let home = env::var("HOME").map_err(|_| "HOME is not set.".to_string())?;
    Ok(PathBuf::from(home).join(".openclaw").join("openclaw.json"))
}

fn default_gateway_port() -> u16 {
    18_789
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use serial_test::serial;
    use tempfile::tempdir;

    use super::{LoadedGatewayConfig, load_gateway_config};

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var(key).ok();
            // SAFETY: These tests are marked `#[serial]`, so the process-wide env var
            // mutation does not race with another test in this suite.
            unsafe {
                env::set_var(key, value);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(previous) => {
                    // SAFETY: These tests are marked `#[serial]`, so the process-wide env var
                    // mutation does not race with another test in this suite.
                    unsafe {
                        env::set_var(self.key, previous);
                    }
                }
                None => {
                    // SAFETY: These tests are marked `#[serial]`, so the process-wide env var
                    // mutation does not race with another test in this suite.
                    unsafe {
                        env::remove_var(self.key);
                    }
                }
            }
        }
    }

    #[test]
    fn loaded_gateway_config_accepts_valid_values() {
        let config = LoadedGatewayConfig::new("test-token", "ws://127.0.0.1:18789/")
            .expect("accept valid gateway config");

        assert_eq!(config.token, "test-token");
        assert_eq!(config.ws_url, "ws://127.0.0.1:18789/");
    }

    #[test]
    fn loaded_gateway_config_rejects_empty_token() {
        let error = LoadedGatewayConfig::new("   ", "ws://127.0.0.1:18789/")
            .expect_err("reject empty token");

        assert!(error.contains("No gateway auth token"));
    }

    #[test]
    fn loaded_gateway_config_rejects_empty_ws_url() {
        let error =
            LoadedGatewayConfig::new("test-token", "   ").expect_err("reject empty websocket url");

        assert!(error.contains("No gateway websocket URL"));
    }

    #[test]
    #[serial]
    fn load_gateway_config_reads_path_from_env_override() {
        let tempdir = tempdir().expect("create tempdir");
        let config_path = tempdir.path().join("openclaw.json");
        fs::write(
            &config_path,
            r#"{
  "gateway": {
    "port": 19000,
    "auth": {
      "token": "test-token"
    }
  }
}"#,
        )
        .expect("write openclaw config");
        let _guard = EnvVarGuard::set(
            "OPENCLAW_CONFIG_PATH",
            config_path.to_str().expect("config path as utf-8"),
        );

        let config = load_gateway_config().expect("load gateway config from env override");

        assert_eq!(config.token, "test-token");
        assert_eq!(config.ws_url, "ws://127.0.0.1:19000/");
    }

    #[test]
    #[serial]
    fn load_gateway_config_reports_missing_token_from_env_override() {
        let tempdir = tempdir().expect("create tempdir");
        let config_path = tempdir.path().join("openclaw.json");
        fs::write(
            &config_path,
            r#"{
  "gateway": {
    "port": 19000,
    "auth": {
      "token": ""
    }
  }
}"#,
        )
        .expect("write openclaw config");
        let _guard = EnvVarGuard::set(
            "OPENCLAW_CONFIG_PATH",
            config_path.to_str().expect("config path as utf-8"),
        );

        let error = load_gateway_config().expect_err("reject empty token from config file");

        assert!(error.contains("No gateway auth token was found"));
        assert!(error.contains("openclaw.json"));
    }
}
