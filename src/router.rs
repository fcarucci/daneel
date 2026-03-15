use dioxus::prelude::*;

use crate::components::layout::AppLayout;
use crate::pages::{
    agents::AgentsPage, dashboard::DashboardPage, not_found::NotFoundPage, settings::SettingsPage,
};

#[derive(Clone, Debug, PartialEq, Eq, Routable)]
pub enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    DashboardPage {},
    #[route("/agents")]
    AgentsPage {},
    #[route("/settings")]
    SettingsPage {},
    #[end_layout]
    #[route("/:..segments")]
    NotFoundPage { segments: Vec<String> },
}

impl Route {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::DashboardPage { .. } => "Dashboard",
            Self::AgentsPage { .. } => "Agents",
            Self::SettingsPage { .. } => "Settings",
            Self::NotFoundPage { .. } => "Not Found",
        }
    }
}
// SPDX-License-Identifier: Apache-2.0
