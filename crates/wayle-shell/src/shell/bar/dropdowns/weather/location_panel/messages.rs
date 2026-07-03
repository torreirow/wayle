use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_weather::{WeatherErrorKind, WeatherService};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PageState {
    Loading,
    Loaded,
    Error,
}

impl PageState {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::Loaded => "loaded",
            Self::Error => "error",
        }
    }
}

pub(crate) struct LocationPanelInit {
    pub weather: Arc<WeatherService>,
    pub config: Arc<ConfigService>,
    pub display_name: Option<String>,
    /// Raw location string (city name or "lat,lon") — used to restart polling on Retry.
    pub location_str: String,
}

#[derive(Debug)]
pub(crate) enum LocationPanelInput {
    Retry,
}

#[derive(Debug)]
pub(crate) enum LocationPanelCmd {
    PageChanged {
        page: PageState,
        error: Option<WeatherErrorKind>,
    },
}
