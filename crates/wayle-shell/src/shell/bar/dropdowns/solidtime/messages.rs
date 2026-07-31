use std::sync::Arc;

use chrono::{DateTime, Utc};
use wayle_config::ConfigService;

pub(crate) struct SolidtimeDropdownInit {
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum SolidtimeDropdownMsg {
    StartTimer,
    StopTimer,
}

#[derive(Debug)]
pub(crate) enum SolidtimeDropdownCmd {
    StatusUpdate {
        running: bool,
        start: Option<DateTime<Utc>>,
        description: String,
        project: Option<String>,
        connected: bool,
    },
    Tick,
    Projects(Vec<String>),
    Recent(Vec<RecentEntry>),
    ScaleChanged(f32),
}

#[derive(Debug, Clone)]
pub(crate) struct RecentEntry {
    pub description: String,
    pub project: Option<String>,
    pub duration_secs: i64,
}
