use std::{rc::Rc, sync::Arc};

use chrono::{DateTime, Utc};
use wayle_config::ConfigService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct SolidtimeInit {
    pub settings: BarSettings,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum SolidtimeMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum SolidtimeCmd {
    StatusUpdate {
        running: bool,
        start: Option<DateTime<Utc>>,
        description: String,
        project: Option<String>,
    },
    Tick,
}
