// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::models::live_gateway::LiveGatewayLevel;
use crate::models::live_gateway::{
    BackendConnectionState, LiveGatewayEvent, OperatorConnectionState,
    resolve_operator_connection_state,
};

#[derive(Clone, Copy)]
pub(crate) struct LiveGatewayState {
    pub live_status: Signal<Option<LiveGatewayEvent>>,
    pub backend_state: Signal<BackendConnectionState>,
}

impl LiveGatewayState {
    pub fn operator_state(&self) -> OperatorConnectionState {
        resolve_operator_connection_state(
            (self.backend_state)(),
            (self.live_status)().map(|event| event.level),
        )
    }

    pub fn is_frozen(&self) -> bool {
        (self.backend_state)() == BackendConnectionState::Disconnected
    }
}

#[component]
pub fn LiveGatewayProvider(children: Element) -> Element {
    let live_gateway = use_live_gateway_state();
    use_context_provider(|| live_gateway);

    rsx! { {children} }
}

pub(crate) fn use_live_gateway() -> LiveGatewayState {
    use_context::<LiveGatewayState>()
}

fn use_live_gateway_state() -> LiveGatewayState {
    let live_status = use_signal(|| None::<LiveGatewayEvent>);
    let backend_state = use_signal(initial_backend_connection_state);
    let _ = non_wasm_reconnect_state_sentinel();

    #[cfg(target_arch = "wasm32")]
    let mut live_listener_attached = use_signal(|| false);

    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            attach_live_gateway_listener(
                &mut live_listener_attached,
                live_status.clone(),
                backend_state.clone(),
            );
        });
    }

    LiveGatewayState {
        live_status,
        backend_state,
    }
}

fn initial_backend_connection_state() -> BackendConnectionState {
    if cfg!(target_arch = "wasm32") {
        BackendConnectionState::Connecting
    } else {
        BackendConnectionState::Connected
    }
}

fn non_wasm_reconnect_state_sentinel() -> Option<BackendConnectionState> {
    if cfg!(target_arch = "wasm32") {
        None
    } else {
        // Keep SSR/test builds aware of the reconnecting state without changing runtime behavior.
        Some(BackendConnectionState::Connecting)
    }
}

#[cfg(target_arch = "wasm32")]
fn attach_live_gateway_listener(
    live_listener_attached: &mut Signal<bool>,
    live_status: Signal<Option<LiveGatewayEvent>>,
    backend_state: Signal<BackendConnectionState>,
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
        let mut backend_state = backend_state.clone();
        move |event: web_sys::MessageEvent| {
            if let Some(parsed) = event
                .data()
                .as_string()
                .and_then(|text| serde_json::from_str::<LiveGatewayEvent>(&text).ok())
            {
                live_status.set(Some(parsed));
                backend_state.set(BackendConnectionState::Connected);
            }
        }
    });
    event_source.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let onopen = Closure::<dyn FnMut(web_sys::Event)>::new({
        let mut backend_state = backend_state.clone();
        let mut live_status = live_status.clone();
        move |_| {
            backend_state.set(BackendConnectionState::Connected);
            if matches!(
                live_status.peek().as_ref().map(|event| event.level),
                None | Some(LiveGatewayLevel::Disconnected)
            ) {
                live_status.set(Some(connecting_event()));
            }
        }
    });
    event_source.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    let onerror = Closure::<dyn FnMut(web_sys::Event)>::new({
        let mut live_status = live_status.clone();
        let mut backend_state = backend_state.clone();
        move |_| {
            backend_state.set(BackendConnectionState::Disconnected);
            live_status.set(Some(disconnected_event()));
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
        level: LiveGatewayLevel::Connecting,
        summary: "Gateway event stream reconnecting.".to_string(),
        detail: "The browser will retry the live event stream automatically.".to_string(),
    }
}

#[cfg(target_arch = "wasm32")]
fn disconnected_event() -> LiveGatewayEvent {
    LiveGatewayEvent {
        level: LiveGatewayLevel::Disconnected,
        summary: "Backend event stream disconnected.".to_string(),
        detail: "The browser is keeping the current view frozen while it retries the backend."
            .to_string(),
    }
}
