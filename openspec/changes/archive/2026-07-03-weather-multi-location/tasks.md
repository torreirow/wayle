## 1. Config Schema

- [x] 1.1 Add `WeatherLocation` struct to `wayle-config/src/schemas/modules/weather/mod.rs` with `location: String` and `name: Option<String>` fields (serde + JsonSchema derives)
- [x] 1.2 Add `locations: ConfigProperty<Vec<WeatherLocation>>` field to `WeatherConfig` with empty default, including API quota warning in the doc comment
- [x] 1.3 Add `WeatherLocation` and `locations`-related i18n keys to `wayle-i18n` settings locale files (en-US and fr) matching the existing `settings-modules-weather-*` prefix

## 2. Bootstrap & Services

- [x] 2.1 Add `effective_locations` helper to `wayle-shell/src/bootstrap/weather.rs` that returns `Vec<(location_query, display_name_override)>` — uses `locations` if non-empty, otherwise falls back to `location` + `location_name`
- [x] 2.2 Refactor `build_weather_service` → `build_weather_services` to return `Vec<Arc<WeatherService>>`, one per effective location entry
- [x] 2.3 Update `ShellServices::weather` type from `Arc<WeatherService>` to `Vec<Arc<WeatherService>>`
- [x] 2.4 Update all call sites that construct `ShellServices` to pass the new `Vec`

## 3. Config Watchers

- [x] 3.1 Update `wayle-shell/src/watchers/weather.rs` — shared config watchers (provider, units, interval, API keys) iterate over all services in the `Vec`
- [x] 3.2 Update location watcher to watch `locations` vec and call `set_location` on each service by index when the vec changes
- [x] 3.3 Keep `spawn_secrets_reload_watcher` working with all services

## 4. WeatherLocationPanel Component

- [x] 4.1 Create `wayle-shell/src/shell/bar/dropdowns/weather/location_panel/mod.rs` — new Relm4 component holding `WeatherHeader`, `StatsGrid`, `HourlyForecast`, `DailyForecast`, `SunTimes` controllers for one `Arc<WeatherService>`
- [x] 4.2 Move the page stack (loading / loaded / error) logic from `WeatherDropdown` into `WeatherLocationPanel`
- [x] 4.3 Move the `WeatherDropdownCmd::PageChanged` command output handling into `WeatherLocationPanel` watchers
- [x] 4.4 Export `WeatherLocationPanel` and its `Init` type from the `location_panel` module

## 5. Weather Dropdown Refactor

- [x] 5.1 Update `WeatherDropdownInit` to accept `Vec<Arc<WeatherService>>` instead of a single service
- [x] 5.2 Replace the five sub-component fields in `WeatherDropdown` with `Vec<Controller<WeatherLocationPanel>>`
- [x] 5.3 Add `active_location: usize` field to `WeatherDropdown` (default 0)
- [x] 5.4 Add location tab bar widget to the dropdown view (horizontal `gtk::Box` with toggle buttons, hidden when `locations.len() == 1`)
- [x] 5.5 Add `gtk::Stack` for location panels with `Crossfade` transition; register each `WeatherLocationPanel` widget as a named child
- [x] 5.6 Add `WeatherDropdownInput::SwitchLocation(usize)` message and handler that sets `active_location` and updates the stack's visible child
- [x] 5.7 Connect tab button click events to `SwitchLocation(i)` input messages
- [x] 5.8 Reset active location to 0 when the dropdown popover is closed (connect `closed` signal)

## 6. Bar Module Update

- [x] 6.1 Update `WeatherInit` in `wayle-shell/src/shell/bar/modules/weather/messages.rs` to accept `Vec<Arc<WeatherService>>`
- [x] 6.2 Update `WeatherModule` init to pass only `services[0]` to the label/icon watchers (existing `spawn_watchers`)
- [x] 6.3 Update `wayle-shell/src/shell/bar/dropdowns/weather/factory.rs` and the module factory to pass the full `Vec<Arc<WeatherService>>` to `WeatherDropdown`

## 7. Styling

- [x] 7.1 Add SCSS rules for the location tab bar in `wayle-styling/scss/modules/weather_dropdown/` (tab bar container, active/inactive tab button states)
