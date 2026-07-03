use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_weather::WeatherService;

pub(crate) struct WeatherHeaderInit {
    pub weather: Arc<WeatherService>,
    pub config: Arc<ConfigService>,
    /// Override the location display name. Takes priority over the config's
    /// `location-name` field. Used by multi-location panels to show per-entry names.
    pub display_name_override: Option<String>,
}

#[derive(Debug)]
pub(crate) enum WeatherHeaderCmd {
    WeatherChanged,
    TickUpdatedAgo,
}
