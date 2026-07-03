## Context

The weather module currently uses a single `Arc<WeatherService>` threaded through `ShellServices`, the bar module, and the dropdown. All sub-components (`WeatherHeader`, `StatsGrid`, `HourlyForecast`, `DailyForecast`, `SunTimes`) accept an `Arc<WeatherService>` at init time.

`WeatherService` (external crate `wayle-weather 0.1.2`) is a fully self-contained polling actor: it owns a Tokio background task, exposes two reactive `Property<T>` fields (`weather`, `status`), and supports runtime reconfiguration via setter methods. Multiple independent instances can coexist without issue.

## Goals / Non-Goals

**Goals:**
- Allow users to configure 2–3 weather locations in a single `[modules.weather]` block
- Display a tab bar in the dropdown when more than one location is configured; clicking a tab switches the visible forecast panel
- Zero breaking changes for users who only configure a single location
- Keep the bar button label bound to the first (primary) location

**Non-Goals:**
- Adding/removing locations at runtime without restart
- Displaying multiple location temperatures simultaneously on the bar
- Syncing active tab state to disk across restarts
- GPS/auto-location detection

## Decisions

### Decision 1: `Vec<Arc<WeatherService>>` in `ShellServices`

**Chosen**: Change `ShellServices::weather` from `Arc<WeatherService>` to `Vec<Arc<WeatherService>>`. Bootstrap reads the effective location list (from `locations` or fallback to `location`) and builds one service per entry.

**Alternative considered**: A `MultiWeatherService` wrapper that manages a `Vec` internally and exposes a switchable interface. Rejected — the indirection adds complexity without benefit, since Relm4 components already manage their own subscriptions.

**Alternative considered**: Keep a single service and call `set_location` to switch. Rejected — would discard cached data on every switch, causing a loading flash each time the user clicks a tab.

### Decision 2: `WeatherLocationPanel` component

**Chosen**: Extract a new `WeatherLocationPanel` Relm4 component that owns the five existing sub-components for one location (header, stats, hourly, daily, sun times). The dropdown holds a `Vec<Controller<WeatherLocationPanel>>` and places each in a `gtk::Stack`. The existing per-location error/loading page logic is handled inside each panel.

**Alternative considered**: Keep the existing flat layout and swap `Arc<WeatherService>` refs when the tab changes. Rejected — Relm4 components cannot change their init-time `Arc` refs; this would require tearing down and rebuilding components on every tab switch.

**Alternative considered**: Duplicate the entire dropdown hierarchy N times. Rejected — the tab bar itself is shared, so clean separation requires a panel component.

### Decision 3: Tab label source

**Chosen**: Tab label = configured `name` override if set, else the raw `location` query string (e.g., "Amsterdam", "48.8566,2.3522"). The API-resolved city name is shown in the panel header (as it is today) but not used for the tab label.

**Rationale**: Using the config-based string is stable and available immediately — no waiting for the first successful fetch. Coordinates display gracefully (user can always set `name = "Parijs"` to override).

### Decision 4: Backwards compatibility strategy

**Chosen**: If `locations` is absent or empty in config, fall back to the single `location` + `location-name` fields. Bootstrap detects this and creates a single-element `Vec`. The bar module, dropdown, and all watchers receive a `Vec<Arc<WeatherService>>` regardless of how many locations are configured.

**Rationale**: Avoids forking code paths — all code paths work with `Vec`. Single-location users see no regression.

### Decision 5: Runtime location add/remove

**Not supported in v1**. The `Vec<Arc<WeatherService>>` length is fixed at startup. Config watchers can update the `location` query on each existing service (hot-reload of the location *string*) but cannot grow or shrink the vec. Users must restart to add/remove locations.

### Decision 6: Active tab state

Tabs default to index 0 (primary location) when the dropdown opens. State is in-memory only; it resets to 0 on dropdown close or app restart. No persistence.

## Risks / Trade-offs

- **Memory**: Each `WeatherService` runs its own poll loop and caches one `Arc<Weather>`. At 3 locations, this is negligible (~3 HTTP requests every 30 min vs 1).
- **API rate limits**: Users with paid providers (VisualCrossing, WeatherAPI) consume 3× their quota. Documented in config schema.
- **`Vec` index stability**: Watchers that update `services[i].set_location(...)` rely on the `locations` vec keeping the same order. Reordering locations in config hot-reload would mis-match services. Acceptable for v1 — the full fix (restart) is documented.
- **Single-location path**: When `Vec.len() == 1`, the dropdown renders no tab bar. A future zero-location error state would require the `Vec` to be non-empty, which bootstrap must guarantee.
