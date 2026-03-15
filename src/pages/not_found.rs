use dioxus::prelude::*;

#[component]
pub fn NotFoundPage(segments: Vec<String>) -> Element {
    let attempted_path = if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    };

    rsx! {
        section { class: "flex flex-col gap-5",
            article { class: "rounded-[2rem] border border-white/10 bg-[var(--panel-bg)] px-6 py-7 shadow-[0_30px_80px_rgba(2,6,23,0.45)] backdrop-blur-2xl sm:px-8",
                p { class: "m-0 text-[0.7rem] font-semibold uppercase tracking-[0.28em] text-[var(--signal)]", "404" }
                h2 { class: "m-0 mt-3 text-3xl font-semibold tracking-[-0.05em] text-white sm:text-4xl", "Route not found" }
                p { class: "m-0 mt-3 text-sm leading-7 text-slate-300 sm:text-base", "No page is registered for {attempted_path}." }
                Link {
                    class: "mt-5 inline-flex items-center text-sm font-medium text-[var(--signal)] transition hover:text-emerald-200",
                    to: crate::router::Route::DashboardPage {},
                    "Return to dashboard"
                }
            }
        }
    }
}
// SPDX-License-Identifier: Apache-2.0
