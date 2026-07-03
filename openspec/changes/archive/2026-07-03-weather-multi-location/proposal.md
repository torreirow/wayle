## Why

The weather module currently supports only a single location. Users who live, work, or travel between multiple places have no way to monitor weather at 2–3 locations from one module — they must either pick one or add duplicate bar buttons via complex workarounds.

## What Changes

- Add a `locations` list to `WeatherConfig` with per-entry `location` and optional `name` fields
- Bootstrap creates one `WeatherService` per configured location instead of a single global service
- The weather dropdown gains a location tab bar when more than one location is configured
- Switching tabs shows the full forecast panel (header, stats, hourly, daily, sun times) for that location
- The bar button label continues to show the primary (first) location — no bar UX changes
- Single-location configuration remains fully backwards compatible — existing `location` and `location-name` fields continue to work unchanged

## Capabilities

### New Capabilities

- `weather-multi-location-config`: Config schema extension — `WeatherLocation` struct and `locations` field in `WeatherConfig`, with fallback to single `location` when `locations` is absent
- `weather-location-tabs`: Dropdown UI — tab bar that appears when ≥2 locations are configured, switches the visible location panel via `gtk::Stack`
- `weather-location-panel`: Composable sub-component wrapping the existing per-location widgets (header, stats grid, hourly/daily forecast, sun times) per `WeatherService` instance

### Modified Capabilities

## Impact

- `wayle-config`: `WeatherConfig` schema (non-breaking, additive)
- `wayle-shell/bootstrap`: `build_weather_service` → returns `Vec<Arc<WeatherService>>`
- `wayle-shell/services`: `ShellServices::weather` type changes from `Arc<WeatherService>` to `Vec<Arc<WeatherService>>`
- `wayle-shell/watchers/weather`: Updated to iterate over all services for shared config (provider, units, interval, keys) and index-based location updates
- `wayle-shell/dropdowns/weather`: New tab bar + `WeatherLocationPanel` component wrapping existing sub-components
- `wayle-shell/modules/weather`: Accept first service in `Vec` for bar label rendering
- `wayle-i18n` / `wayle-shell/locales`: No new strings required for v1 tabs (labels come from config)
