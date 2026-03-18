// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

use crate::gateway::{connect_gateway, load_gateway_config};

pub(super) fn require_payload<'a>(
    payload: Option<&'a Value>,
    context: &str,
) -> Result<&'a Value, String> {
    payload.ok_or_else(|| format!("{context} did not include a payload."))
}

pub(super) async fn fetch_connect_payload(request_id: &str) -> Result<Value, String> {
    let config = load_gateway_config()?;
    let (mut socket, connect_frame) = connect_gateway(&config, request_id).await?;
    let _ = socket.close(None).await;

    require_payload(connect_frame.payload.as_ref(), "Gateway connect response").cloned()
}
