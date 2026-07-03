use std::{sync::Arc, time::Duration};

use wayle_config::{
    schemas::modules::{ModulesConfig, TemperatureUnit as CfgTempUnit, WeatherProvider},
    secrets,
};
use wayle_weather::{LocationQuery, TemperatureUnit, WeatherProviderKind, WeatherService};

pub(super) fn build_weather_services(modules: &ModulesConfig) -> Vec<Arc<WeatherService>> {
    let cfg = &modules.weather;

    let provider = match cfg.provider.get() {
        WeatherProvider::OpenMeteo => WeatherProviderKind::OpenMeteo,
        WeatherProvider::VisualCrossing => WeatherProviderKind::VisualCrossing,
        WeatherProvider::WeatherApi => WeatherProviderKind::WeatherApi,
    };
    let units = match cfg.units.get() {
        CfgTempUnit::Metric => TemperatureUnit::Metric,
        CfgTempUnit::Imperial => TemperatureUnit::Imperial,
    };
    let poll_interval = Duration::from_secs(u64::from(cfg.refresh_interval_seconds.get()));
    let vc_key = secrets::resolve(cfg.visual_crossing_key.get());
    let wa_key = secrets::resolve(cfg.weatherapi_key.get());

    effective_locations(modules)
        .into_iter()
        .map(|(location_query, _display_name)| {
            let mut builder = WeatherService::builder()
                .poll_interval(poll_interval)
                .provider(provider)
                .location(parse_location(&location_query))
                .units(units);

            if let Some(key) = vc_key.clone() {
                builder = builder.visual_crossing_key(key);
            }
            if let Some(key) = wa_key.clone() {
                builder = builder.weatherapi_key(key);
            }

            Arc::new(builder.build())
        })
        .collect()
}

/// Returns `(location_query, display_name_override)` pairs for all configured locations.
///
/// Uses `locations` list when non-empty; falls back to the single `location` + `location-name`.
pub(crate) fn effective_locations(modules: &ModulesConfig) -> Vec<(String, Option<String>)> {
    let cfg = &modules.weather;
    let list = cfg.locations.get();

    if !list.is_empty() {
        list.iter()
            .map(|entry| (entry.location.clone(), entry.name.clone()))
            .collect()
    } else {
        vec![(
            cfg.location.get().clone(),
            cfg.location_name.get().clone(),
        )]
    }
}

pub(super) fn parse_location(location: &str) -> LocationQuery {
    location
        .split_once(',')
        .and_then(|(lat, lon)| {
            let lat = lat.trim().parse().ok()?;
            let lon = lon.trim().parse().ok()?;
            Some(LocationQuery::coords(lat, lon))
        })
        .unwrap_or_else(|| LocationQuery::city(location))
}
