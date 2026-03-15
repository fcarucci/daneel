use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    rsx! {
        section { class: "flex flex-col gap-5",
            div { class: "flex flex-col gap-2",
                p { class: "m-0 text-[0.7rem] font-semibold uppercase tracking-[0.28em] text-[var(--signal)]", "Control Surface" }
                p { class: "m-0 max-w-2xl text-sm leading-7 text-slate-300 sm:text-base", "Manage gateway connection details, interface preferences, and operational defaults from one place." }
            }
            div { class: "grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]",
                article { class: "rounded-[1.6rem] border border-white/10 bg-[var(--panel-bg)] p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl",
                    h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-white", "Planned controls" }
                    p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "This page will surface gateway endpoint configuration, theme preferences, and device-aware operational settings once persistence is wired in." }
                }
                article { class: "rounded-[1.6rem] border border-white/10 bg-white/6 p-6 shadow-[0_24px_64px_rgba(2,6,23,0.35)] backdrop-blur-xl",
                    h3 { class: "m-0 text-lg font-semibold tracking-[-0.03em] text-white", "Status" }
                    p { class: "m-0 mt-3 text-sm leading-6 text-slate-300", "The shell is ready for typed settings models and server-backed save flows." }
                }
            }
        }
    }
}
// SPDX-License-Identifier: Apache-2.0
