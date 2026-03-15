use dioxus::prelude::*;

use crate::components::{navbar::TopBar, sidebar::Sidebar};
use crate::router::Route;

#[component]
pub fn AppLayout() -> Element {
    rsx! {
        div { class: "min-h-screen bg-[var(--app-bg)] text-[var(--ink-0)]",
            div { class: "grid min-h-screen grid-cols-1 lg:grid-cols-[15.5rem_minmax(0,1fr)]",
                Sidebar {}
                div { class: "min-w-0",
                TopBar {}
                    main { class: "px-5 pb-8 pt-6 sm:px-8 lg:px-10",
                        Outlet::<Route> {}
                    }
                }
            }
        }
    }
}
// SPDX-License-Identifier: Apache-2.0
