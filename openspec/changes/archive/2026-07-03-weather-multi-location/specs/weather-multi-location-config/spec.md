## ADDED Requirements

### Requirement: WeatherLocation struct
The system SHALL provide a `WeatherLocation` config struct with a required `location` string field (city name or `"lat,lon"` coordinates) and an optional `name` string field for overriding the display name shown in the dropdown tab and header.

#### Scenario: Location entry without name override
- **WHEN** a `WeatherLocation` is configured with only `location = "Amsterdam"`
- **THEN** the system SHALL use the API-resolved city name in the header and the raw query string `"Amsterdam"` as the tab label

#### Scenario: Location entry with name override
- **WHEN** a `WeatherLocation` is configured with `location = "48.8566,2.3522"` and `name = "Parijs"`
- **THEN** the system SHALL display `"Parijs"` as the tab label and as the location name in the dropdown header

### Requirement: locations list field in WeatherConfig
The system SHALL add an optional `locations` field (serialized as `locations`) of type `Vec<WeatherLocation>` to `WeatherConfig`. When `locations` contains one or more entries, the system SHALL use those entries as the effective location list and ignore the top-level `location` and `location-name` fields.

#### Scenario: locations field present and non-empty
- **WHEN** the user sets `locations = [{ location = "Amsterdam" }, { location = "San Francisco" }]`
- **THEN** the system SHALL create two `WeatherService` instances, one per entry, at bootstrap

#### Scenario: locations field absent or empty
- **WHEN** the user does not set `locations` (or sets it to an empty list)
- **THEN** the system SHALL fall back to the existing `location` and `location-name` fields, creating exactly one `WeatherService`

### Requirement: Effective location list always non-empty
Bootstrap SHALL guarantee that the effective location list contains at least one entry. If both `locations` is empty/absent and `location` is not set, the system SHALL use the configured default location value.

#### Scenario: No location configured
- **WHEN** neither `locations` nor `location` is explicitly set by the user
- **THEN** the system SHALL use the schema default location (e.g. `"San Francisco"`) and create one `WeatherService`

### Requirement: API rate limit documentation
The config schema documentation for `locations` SHALL warn users that each additional location consumes one additional API slot per polling interval, which may affect quota limits on paid providers.

#### Scenario: Schema documentation
- **WHEN** a user reads the generated JSON schema or settings documentation for `locations`
- **THEN** the docs SHALL include a note about API quota implications for paid providers (VisualCrossing, WeatherAPI)
