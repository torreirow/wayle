use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_weather::WeatherService;

pub(crate) struct WeatherDropdownInit {
    pub weather: Vec<Arc<WeatherService>>,
    pub config: Arc<ConfigService>,
    /// Display label per location (tab label). One entry per WeatherService.
    pub tab_labels: Vec<String>,
    /// Raw location strings (city or "lat,lon") per WeatherService. Used for Retry.
    pub location_strs: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum WeatherDropdownInput {
    Retry,
    SwitchLocation(usize),
}

#[derive(Debug)]
pub(crate) enum WeatherDropdownCmd {
    ScaleChanged(f32),
}
