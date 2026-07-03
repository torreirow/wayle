use wayle_weather::{LocationQuery, WeatherErrorKind};

use super::LocationPanel;
use crate::i18n::t;

impl LocationPanel {
    pub(super) fn error_description(&self) -> String {
        match &self.error_kind {
            Some(WeatherErrorKind::ApiKeyMissing { provider }) => {
                t!(
                    "dropdown-weather-error-api-key",
                    provider = provider.as_str()
                )
            }
            Some(WeatherErrorKind::LocationNotFound { query }) => {
                t!("dropdown-weather-error-location", query = query.as_str())
            }
            Some(WeatherErrorKind::Network) => t!("dropdown-weather-error-network"),
            Some(WeatherErrorKind::RateLimited) => t!("dropdown-weather-error-rate-limit"),
            Some(WeatherErrorKind::Other) | None => t!("dropdown-weather-error-unknown"),
        }
    }

    pub(super) fn trigger_refresh(&self) {
        let query = parse_location(&self.location_str);
        self.weather.set_location(query);
    }
}

fn parse_location(location: &str) -> LocationQuery {
    location
        .split_once(',')
        .and_then(|(lat, lon)| {
            let lat = lat.trim().parse().ok()?;
            let lon = lon.trim().parse().ok()?;
            Some(LocationQuery::coords(lat, lon))
        })
        .unwrap_or_else(|| LocationQuery::city(location))
}
