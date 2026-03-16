// SPDX-License-Identifier: Apache-2.0

use dioxus::prelude::*;

use crate::components::layout::AppLayout;
use crate::pages::{agents::Agents, dashboard::Dashboard, not_found::NotFound, settings::Settings};

#[derive(Clone, Debug, PartialEq, Eq, Routable)]
pub enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Dashboard {},
    #[route("/agents")]
    Agents {},
    #[route("/settings")]
    Settings {},
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

impl Route {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Dashboard { .. } => "Dashboard",
            Self::Agents { .. } => "Agents",
            Self::Settings { .. } => "Settings",
            Self::NotFound { .. } => "Not Found",
        }
    }
}
