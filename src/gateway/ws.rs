// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use serde_json::Value;

#[cfg(feature = "server")]
use super::config::LoadedGatewayConfig;
#[cfg(feature = "server")]
use super::parse::find_string;

#[cfg(feature = "server")]
use serde_json::json;

#[derive(Debug, Deserialize)]
pub(crate) struct Envelope {
    #[serde(rename = "type")]
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) id: Option<String>,
    #[serde(default)]
    pub(crate) ok: Option<bool>,
    #[serde(default)]
    pub(crate) payload: Option<Value>,
    #[serde(default)]
    pub(crate) error: Option<Value>,
}

#[cfg(feature = "server")]
pub(crate) fn connect_request(request_id: &str, token: &str) -> Value {
    json!({
        "type": "req",
        "id": request_id,
        "method": "connect",
        "params": {
            "minProtocol": 3,
            "maxProtocol": 3,
            "role": "operator",
            "scopes": ["operator.read"],
            "client": {
                "id": "gateway-client",
                "version": env!("CARGO_PKG_VERSION"),
                "platform": std::env::consts::OS,
                "mode": "backend"
            },
            "auth": {
                "token": token
            }
        }
    })
}

#[cfg(feature = "server")]
pub(crate) async fn connect_gateway(
    config: &LoadedGatewayConfig,
    request_id: &str,
) -> Result<
    (
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Envelope,
    ),
    String,
> {
    use futures_util::SinkExt;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (mut socket, _) = connect_async(config.ws_url.as_str())
        .await
        .map_err(|error| {
            format!(
                "Could not open gateway websocket {}: {error}",
                config.ws_url
            )
        })?;

    socket
        .send(Message::Text(
            connect_request(request_id, &config.token)
                .to_string()
                .into(),
        ))
        .await
        .map_err(|error| format!("Could not send gateway connect request: {error}"))?;

    let connect_frame = wait_for_response(&mut socket, request_id).await?;
    Ok((socket, connect_frame))
}

#[cfg(feature = "server")]
pub(crate) async fn request_gateway(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    request_id: &str,
    method: &str,
    params: Value,
) -> Result<Envelope, String> {
    use futures_util::SinkExt;
    use tokio_tungstenite::tungstenite::Message;

    socket
        .send(Message::Text(
            json!({
                "type": "req",
                "id": request_id,
                "method": method,
                "params": params,
            })
            .to_string()
            .into(),
        ))
        .await
        .map_err(|error| format!("Could not send gateway {method} request: {error}"))?;

    wait_for_response(socket, request_id).await
}

#[cfg(feature = "server")]
pub(crate) async fn wait_for_response<
    S: futures_util::Stream<
            Item = Result<
                tokio_tungstenite::tungstenite::Message,
                tokio_tungstenite::tungstenite::Error,
            >,
        > + Unpin,
>(
    socket: &mut S,
    expected_id: &str,
) -> Result<Envelope, String> {
    use futures_util::StreamExt;
    use tokio::time::{Duration, timeout};
    use tokio_tungstenite::tungstenite::Message;

    loop {
        let frame = timeout(Duration::from_secs(10), socket.next())
            .await
            .map_err(|_| format!("Timed out waiting for gateway response {expected_id}."))?
            .ok_or_else(|| {
                format!("Gateway closed the socket before responding to {expected_id}.")
            })?
            .map_err(|error| {
                format!("Gateway websocket error while waiting for {expected_id}: {error}")
            })?;

        let Message::Text(text) = frame else {
            continue;
        };

        let envelope: Envelope = serde_json::from_str(&text)
            .map_err(|error| format!("Could not parse gateway response frame: {error}"))?;

        if envelope.kind == "res" && envelope.id.as_deref() == Some(expected_id) {
            if envelope.ok.unwrap_or(false) {
                return Ok(envelope);
            }

            let detail = envelope
                .error
                .as_ref()
                .and_then(|value| find_string(value, &["message", "code", "type"]))
                .unwrap_or_else(|| "Unknown gateway error".to_string());
            return Err(format!("Gateway request {expected_id} failed: {detail}"));
        }
    }
}
