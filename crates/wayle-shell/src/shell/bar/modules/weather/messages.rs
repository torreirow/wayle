use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_weather::WeatherService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct WeatherInit {
    pub settings: BarSettings,
    /// All configured weather services. The bar label tracks only the first (index 0).
    pub weather: Vec<Arc<WeatherService>>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum WeatherMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum WeatherCmd {
    UpdateLabel(String),
    UpdateIcon(String),
}
