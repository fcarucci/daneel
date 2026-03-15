mod components;
mod gateway;
mod models;
mod pages;
mod router;

use dioxus::prelude::*;

#[cfg(feature = "server")]
fn main() {
    dioxus::LaunchBuilder::server().launch(App);
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
// SPDX-License-Identifier: Apache-2.0
