## ADDED Requirements

### Requirement: WeatherLocationPanel component
The system SHALL introduce a `WeatherLocationPanel` Relm4 component that encapsulates the five existing per-location sub-components: `WeatherHeader`, `StatsGrid`, `HourlyForecast`, `DailyForecast`, and `SunTimes`. Each panel instance owns exactly one `Arc<WeatherService>`.

#### Scenario: Panel initialization
- **WHEN** a `WeatherLocationPanel` is initialized with an `Arc<WeatherService>`
- **THEN** it SHALL initialize all five sub-components using that service and arrange them in a vertical box, identical to the current single-location layout

### Requirement: Per-panel loading and error state
Each `WeatherLocationPanel` SHALL manage its own loading/loaded/error page stack independently, based on the status of its bound `WeatherService`. A loading or error state on one panel SHALL not affect the display of other panels.

#### Scenario: One location fails, another loads successfully
- **WHEN** location 0 enters an error state and location 1 loads successfully
- **THEN** the panel for location 0 SHALL show the error view and the panel for location 1 SHALL show the loaded forecast view

#### Scenario: Panel shows loading on initial fetch
- **WHEN** a `WeatherLocationPanel` is first shown and its service has not yet completed the initial fetch
- **THEN** the panel SHALL display the loading state (spinner/loading icon)

### Requirement: WeatherDropdown holds a Vec of panels
The `WeatherDropdown` component SHALL hold a `Vec<Controller<WeatherLocationPanel>>`, one per configured location. The dropdown SHALL no longer directly hold `WeatherHeader`, `StatsGrid`, `HourlyForecast`, `DailyForecast`, or `SunTimes` as top-level fields; those are encapsulated inside each panel.

#### Scenario: Multiple locations at init
- **WHEN** `WeatherDropdown` is initialized with two `Arc<WeatherService>` instances
- **THEN** it SHALL create two `WeatherLocationPanel` controllers and register both in its `gtk::Stack`

### Requirement: Bar module uses first service only
The `WeatherModule` bar button component SHALL continue to receive the full `Vec<Arc<WeatherService>>` but SHALL bind its label and icon watchers only to the first service (index 0). The bar button appearance SHALL be unaffected by additional configured locations.

#### Scenario: Bar label with multiple locations configured
- **WHEN** three locations are configured
- **THEN** the bar button label SHALL reflect the weather of location 0 only

### Requirement: Shared config watchers cover all services
Watchers for shared config fields (provider, units, refresh interval, API keys) SHALL iterate over all `WeatherService` instances in the `Vec` and apply the change to each. Watchers for location-specific fields SHALL update the matching service by index.

#### Scenario: Provider change applies to all services
- **WHEN** the user changes `provider` in the config at runtime
- **THEN** all configured `WeatherService` instances SHALL switch to the new provider and restart their polling loops

#### Scenario: Location string hot-reload
- **WHEN** the user changes the `location` string of the second entry in `locations` at runtime
- **THEN** the `WeatherService` at index 1 SHALL call `set_location` with the new query and restart its polling loop
