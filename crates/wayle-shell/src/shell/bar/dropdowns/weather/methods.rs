use super::WeatherDropdown;

impl WeatherDropdown {
    pub(super) fn active_panel_name(&self) -> String {
        format!("location-{}", self.active_location)
    }
}
