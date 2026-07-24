use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use wayle_derive::wayle_config;

use crate::{
    ConfigProperty,
    docs::{ConfigGroup, ModuleInfo, ModuleInfoProvider},
};

/// Planify task list dropdown configuration.
#[wayle_config]
pub struct PlanifyConfig {
    /// Planify project name to display tasks from.
    #[default(String::from("Inbox"))]
    pub project: ConfigProperty<String>,

    /// Path to the Planify SQLite database file.
    ///
    /// Leave unset to use the default XDG path:
    /// `~/.local/share/io.github.alainm23.planify/database.db`
    #[serde(rename = "db-path")]
    #[default(None)]
    pub db_path: ConfigProperty<Option<String>>,
}

impl ModuleInfoProvider for PlanifyConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("planify"),
            schema: || schema_for!(PlanifyConfig),
            layout_id: None,
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        vec![]
    }
}

crate::register_module!(PlanifyConfig);
