use std::sync::Arc;

use wayle_config::ConfigService;

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    /// Raw line from `cliphist list`: "id\tcontent"
    pub raw: String,
    /// Truncated display text for the UI row.
    pub preview: String,
}

pub struct ClipboardDropdownInit {
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub enum ClipboardDropdownMsg {
    Refresh,
    EntrySelected(i32),
}

#[derive(Debug)]
pub enum ClipboardDropdownCmd {
    Entries(Vec<ClipboardEntry>),
    ScaleChanged(f32),
}
