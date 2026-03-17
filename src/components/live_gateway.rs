// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::models::live_gateway::LiveGatewayEvent;

pub(crate) fn use_live_gateway_events() -> (Signal<Option<LiveGatewayEvent>>, Signal<bool>) {
    let live_status = use_signal(|| None::<LiveGatewayEvent>);
    let live_seen = use_signal(|| false);

    #[cfg(target_arch = "wasm32")]
    let mut live_listener_attached = use_signal(|| false);

    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            attach_live_gateway_listener(
                &mut live_listener_attached,
                live_status.clone(),
                live_seen.clone(),
            );
        });
    }

    (live_status, live_seen)
}

#[cfg(target_arch = "wasm32")]
fn attach_live_gateway_listener(
    live_listener_attached: &mut Signal<bool>,
    live_status: Signal<Option<LiveGatewayEvent>>,
    live_seen: Signal<bool>,
) {
    use web_sys::wasm_bindgen::{JsCast, closure::Closure};

    if !live_stream_enabled() || *live_listener_attached.peek() {
        return;
    }
    live_listener_attached.set(true);

    let event_source = web_sys::EventSource::new("/api/gateway/events")
        .expect("create EventSource for gateway events");

    let onmessage = Closure::<dyn FnMut(web_sys::MessageEvent)>::new({
        let mut live_status = live_status.clone();
        let mut live_seen = live_seen.clone();
        move |event: web_sys::MessageEvent| {
            if let Some(parsed) = event
                .data()
                .as_string()
                .and_then(|text| serde_json::from_str::<LiveGatewayEvent>(&text).ok())
            {
                live_status.set(Some(parsed));
                live_seen.set(true);
            }
        }
    });
    event_source.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let onerror = Closure::<dyn FnMut(web_sys::Event)>::new({
        let mut live_status = live_status.clone();
        let mut live_seen = live_seen.clone();
        move |_| {
            live_status.set(Some(connecting_event()));
            live_seen.set(false);
        }
    });
    event_source.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();
    std::mem::forget(event_source);
}

#[cfg(target_arch = "wasm32")]
fn live_stream_enabled() -> bool {
    web_sys::window()
        .and_then(|window| window.location().search().ok())
        .map(|query| !query.contains("e2e-disable-live=1"))
        .unwrap_or(true)
}

#[cfg(target_arch = "wasm32")]
fn connecting_event() -> LiveGatewayEvent {
    LiveGatewayEvent {
        level: crate::models::live_gateway::LiveGatewayLevel::Connecting,
        summary: "Gateway event stream reconnecting.".to_string(),
        detail: "The browser will retry the live event stream automatically.".to_string(),
    }
}
