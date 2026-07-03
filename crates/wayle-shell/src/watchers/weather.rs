//! Weather service hot-reload watcher.

use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use wayle_config::{
    ConfigService,
    schemas::modules::{TemperatureUnit, WeatherConfig, WeatherProvider},
    secrets,
};
use wayle_weather::WeatherService;

use crate::shell::ShellServices;

pub fn spawn(services: &ShellServices) {
    let config_service = services.config.clone();
    let weather = services.weather.clone();
    let config = config_service.config().clone();
    let weather_config = &config.modules.weather;

    spawn_location_watcher(weather_config, &weather);
    spawn_provider_watcher(weather_config, &weather);
    spawn_units_watcher(weather_config, &weather);
    spawn_interval_watcher(weather_config, &weather);
    spawn_visual_crossing_key_watcher(weather_config, &weather);
    spawn_weatherapi_key_watcher(weather_config, &weather);
    spawn_secrets_reload_watcher(&config_service, weather_config, &weather);
}

fn spawn_location_watcher(config: &WeatherConfig, services: &Vec<Arc<WeatherService>>) {
    let mut stream = config.locations.watch();
    let services = services.clone();
    let single_location = config.location.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(locations) = stream.next().await {
            if locations.is_empty() {
                // Fallback: apply the single-location field to all services (index 0)
                if let Some(svc) = services.first() {
                    svc.set_location(parse_location(&single_location.get()));
                }
            } else {
                for (i, entry) in locations.iter().enumerate() {
                    if let Some(svc) = services.get(i) {
                        svc.set_location(parse_location(&entry.location));
                    }
                }
            }
        }
    });
}

fn spawn_provider_watcher(config: &WeatherConfig, services: &Vec<Arc<WeatherService>>) {
    let mut stream = config.provider.watch();
    let services = services.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(provider) = stream.next().await {
            let kind = match provider {
                WeatherProvider::OpenMeteo => wayle_weather::WeatherProviderKind::OpenMeteo,
                WeatherProvider::VisualCrossing => {
                    wayle_weather::WeatherProviderKind::VisualCrossing
                }
                WeatherProvider::WeatherApi => wayle_weather::WeatherProviderKind::WeatherApi,
            };
            for svc in &services {
                svc.set_provider(kind);
            }
        }
    });
}

fn spawn_units_watcher(config: &WeatherConfig, services: &Vec<Arc<WeatherService>>) {
    let mut stream = config.units.watch();
    let services = services.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(units) = stream.next().await {
            let units = match units {
                TemperatureUnit::Metric => wayle_weather::TemperatureUnit::Metric,
                TemperatureUnit::Imperial => wayle_weather::TemperatureUnit::Imperial,
            };
            for svc in &services {
                svc.set_units(units);
            }
        }
    });
}

fn spawn_interval_watcher(config: &WeatherConfig, services: &Vec<Arc<WeatherService>>) {
    let mut stream = config.refresh_interval_seconds.watch();
    let services = services.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(interval_secs) = stream.next().await {
            let dur = Duration::from_secs(u64::from(interval_secs));
            for svc in &services {
                svc.set_poll_interval(dur);
            }
        }
    });
}

fn spawn_visual_crossing_key_watcher(config: &WeatherConfig, services: &Vec<Arc<WeatherService>>) {
    let mut stream = config.visual_crossing_key.watch();
    let services = services.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(key) = stream.next().await {
            let resolved = secrets::resolve(key);
            for svc in &services {
                svc.set_visual_crossing_key(resolved.clone());
            }
        }
    });
}

fn spawn_weatherapi_key_watcher(config: &WeatherConfig, services: &Vec<Arc<WeatherService>>) {
    let mut stream = config.weatherapi_key.watch();
    let services = services.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(key) = stream.next().await {
            let resolved = secrets::resolve(key);
            for svc in &services {
                svc.set_weatherapi_key(resolved.clone());
            }
        }
    });
}

fn parse_location(location: &str) -> wayle_weather::LocationQuery {
    location
        .split_once(',')
        .and_then(|(lat, lon)| {
            let lat = lat.trim().parse().ok()?;
            let lon = lon.trim().parse().ok()?;
            Some(wayle_weather::LocationQuery::coords(lat, lon))
        })
        .unwrap_or_else(|| wayle_weather::LocationQuery::city(location))
}

fn spawn_secrets_reload_watcher(
    config_service: &Arc<ConfigService>,
    weather_config: &WeatherConfig,
    services: &Vec<Arc<WeatherService>>,
) {
    let Some(mut rx) = config_service.subscribe_secrets_reload() else {
        return;
    };

    let vc_key = weather_config.visual_crossing_key.clone();
    let wa_key = weather_config.weatherapi_key.clone();
    let services = services.clone();

    tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            let vc = secrets::resolve(vc_key.get());
            let wa = secrets::resolve(wa_key.get());
            for svc in &services {
                svc.set_visual_crossing_key(vc.clone());
                svc.set_weatherapi_key(wa.clone());
            }
        }
    });
}
