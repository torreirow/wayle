## ADDED Requirements

### Requirement: Tab bar visibility
The weather dropdown SHALL display a horizontal tab bar above the location panel content if and only if two or more locations are configured. When exactly one location is configured, no tab bar SHALL be rendered (preserving the current single-location appearance).

#### Scenario: Single location configured
- **WHEN** the effective location list contains exactly one entry
- **THEN** the dropdown SHALL render the location panel without any tab bar

#### Scenario: Multiple locations configured
- **WHEN** the effective location list contains two or three entries
- **THEN** the dropdown SHALL render a tab bar with one button per location above the content area

### Requirement: Tab label content
Each tab button's label SHALL be the configured `name` override for that location entry if set, otherwise the raw `location` query string (e.g., `"Amsterdam"`, `"48.8566,2.3522"`).

#### Scenario: Named location entry
- **WHEN** a location entry has `name = "SF"` configured
- **THEN** the corresponding tab button SHALL display the label `"SF"`

#### Scenario: Unnamed location entry
- **WHEN** a location entry has no `name` and `location = "San Francisco"`
- **THEN** the corresponding tab button SHALL display the label `"San Francisco"`

### Requirement: Active tab selection
The system SHALL track an active location index. Clicking a tab SHALL set the active index to the corresponding location and reveal the matching location panel. The first location (index 0) SHALL be active by default when the dropdown opens.

#### Scenario: Initial dropdown open
- **WHEN** the user opens the weather dropdown
- **THEN** the tab for index 0 (primary location) SHALL be visually active and its panel SHALL be visible

#### Scenario: User clicks a tab
- **WHEN** the user clicks the tab for location index N
- **THEN** the tab for index N SHALL become visually active, the panel for index N SHALL become visible, and all other panels SHALL be hidden

### Requirement: Location panel switching via gtk::Stack
The dropdown SHALL use a `gtk::Stack` with `Crossfade` transition to switch between location panels. Each location panel SHALL be added as a named child of the stack at init time.

#### Scenario: Tab switch animation
- **WHEN** the user clicks a different location tab
- **THEN** the stack SHALL transition to the corresponding child using the Crossfade animation

### Requirement: Tab state resets on dropdown close
The active tab index SHALL reset to 0 each time the dropdown is re-opened. Tab selection state is not persisted across dropdown open/close cycles or application restarts.

#### Scenario: Dropdown reopened after tab switch
- **WHEN** the user closes the dropdown after switching to tab 2, then reopens the dropdown
- **THEN** tab 0 (primary location) SHALL be active and visible
