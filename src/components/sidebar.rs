// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::router::Route;

const NAV_ITEMS: [Route; 3] = [Route::Dashboard {}, Route::Agents {}, Route::Settings {}];

#[component]
pub fn Sidebar() -> Element {
    let route: Route = use_route();

    rsx! {
        aside { class: "border-b border-white/10 bg-[var(--rail-bg)]/88 px-4 py-6 backdrop-blur-2xl lg:border-b-0 lg:border-r lg:px-5", "data-sidebar-polish": "enhanced",
            div { class: "mb-7 flex flex-col gap-2",
                p { class: "page-kicker m-0 text-[0.7rem] font-semibold uppercase tracking-[0.28em] text-[var(--signal)]", "OpenClaw" }
                h1 { class: "m-0 text-[1.95rem] font-semibold tracking-[-0.05em] text-white", "Daneel" }
                p { class: "m-0 max-w-[13rem] text-sm leading-6 text-slate-300", "A focused mission control surface for operators." }
            }
            nav { class: "flex flex-col gap-2",
                for nav_route in NAV_ITEMS {
                    {
                        let is_active = nav_route == route;
                        let class_name = if is_active {
                            "polish-card polish-card--interactive flex items-center gap-3 rounded-2xl border border-emerald-300/20 bg-emerald-300/10 px-4 py-3 text-[1.05rem] font-medium text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] transition hover:translate-x-0.5 hover:border-emerald-200/30 hover:bg-emerald-300/12"
                        } else {
                            "polish-card polish-card--interactive flex items-center gap-3 rounded-2xl border border-transparent px-4 py-3 text-[1.05rem] font-medium text-slate-300 transition hover:translate-x-0.5 hover:border-white/10 hover:bg-white/5 hover:text-white"
                        };
                        let icon_class = if is_active {
                            "h-[0.95rem] w-[0.95rem] shrink-0 text-emerald-200"
                        } else {
                            "h-[0.95rem] w-[0.95rem] shrink-0 text-slate-400"
                        };

                        rsx! {
                            Link {
                                class: class_name,
                                "data-nav-polish": if is_active { "active" } else { "idle" },
                                to: nav_route.clone(),
                                NavIcon { route: nav_route.clone(), class: icon_class }
                                span { "{nav_route.label()}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavIcon(route: Route, class: &'static str) -> Element {
    match route {
        Route::Dashboard {} => rsx! {
            svg {
                class,
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                rect { x: "3", y: "3", width: "7", height: "7", rx: "1.5" }
                rect { x: "14", y: "3", width: "7", height: "4", rx: "1.5" }
                rect { x: "14", y: "10", width: "7", height: "11", rx: "1.5" }
                rect { x: "3", y: "13", width: "7", height: "8", rx: "1.5" }
            }
        },
        Route::Agents {} => rsx! {
            svg {
                class,
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                circle { cx: "9", cy: "8", r: "2.5" }
                path { d: "M4.5 17.5c.8-2.4 2.7-3.8 4.5-3.8s3.7 1.4 4.5 3.8" }
                circle { cx: "17.5", cy: "9.5", r: "2" }
                path { d: "M15.5 17c.5-1.6 1.8-2.7 3.5-2.7 1.1 0 2 .4 2.5 1.2" }
            }
        },
        Route::Settings {} => rsx! {
            svg {
                class,
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                circle { cx: "12", cy: "12", r: "3" }
                path { d: "M19.4 15a1 1 0 0 0 .2 1.1l.1.1a2 2 0 0 1 0 2.8 2 2 0 0 1-2.8 0l-.1-.1a1 1 0 0 0-1.1-.2 1 1 0 0 0-.6.9V20a2 2 0 0 1-4 0v-.2a1 1 0 0 0-.6-.9 1 1 0 0 0-1.1.2l-.1.1a2 2 0 0 1-2.8 0 2 2 0 0 1 0-2.8l.1-.1a1 1 0 0 0 .2-1.1 1 1 0 0 0-.9-.6H4a2 2 0 0 1 0-4h.2a1 1 0 0 0 .9-.6 1 1 0 0 0-.2-1.1l-.1-.1a2 2 0 0 1 0-2.8 2 2 0 0 1 2.8 0l.1.1a1 1 0 0 0 1.1.2 1 1 0 0 0 .6-.9V4a2 2 0 0 1 4 0v.2a1 1 0 0 0 .6.9 1 1 0 0 0 1.1-.2l.1-.1a2 2 0 0 1 2.8 0 2 2 0 0 1 0 2.8l-.1.1a1 1 0 0 0-.2 1.1 1 1 0 0 0 .9.6H20a2 2 0 0 1 0 4h-.2a1 1 0 0 0-.9.6z" }
            }
        },
        _ => rsx! {
            svg {
                class,
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.8",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                circle { cx: "12", cy: "12", r: "8" }
            }
        },
    }
}
