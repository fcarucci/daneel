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

#[derive(Debug)]
pub(crate) struct LoadedGatewayConfig {
    pub(crate) token: String,
    pub(crate) ws_url: String,
}

pub(crate) const DEFAULT_GATEWAY_URL: &str = "ws://127.0.0.1:18789";

pub(crate) fn load_gateway_config() -> Result<LoadedGatewayConfig, String> {
    let path = openclaw_config_path()?;
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let parsed: OpenClawConfig = serde_json::from_str(&raw)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))?;

    if parsed.gateway.auth.token.is_empty() {
        return Err(format!(
            "No gateway auth token was found in {}.",
            path.display()
        ));
    }

    Ok(LoadedGatewayConfig {
        token: parsed.gateway.auth.token,
        ws_url: format!("ws://127.0.0.1:{}/", parsed.gateway.port),
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
