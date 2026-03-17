// SPDX-License-Identifier: Apache-2.0

mod adapter;
mod app_state;
mod client;
mod components;
mod gateway;
mod live;
mod models;
mod pages;
mod router;
#[cfg(test)]
mod test_support;
mod utils;

use dioxus::prelude::*;

#[cfg(feature = "server")]
fn main() {
    dioxus_server::serve(|| async move {
        let _ = app_state::warm_server_app_state();
        let hub = live::init_live_hub();
        tokio::spawn(live::run_gateway_event_bridge(hub));

        let router = live::router().merge(dioxus_server::router(App));

        Ok(router)
    });
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        Router::<router::Route> {}
    }
}
