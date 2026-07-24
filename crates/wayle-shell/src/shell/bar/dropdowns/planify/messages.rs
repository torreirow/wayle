use std::sync::Arc;

use chrono::NaiveDate;
use wayle_config::ConfigService;

#[derive(Debug, Clone)]
pub struct PlanifyTask {
    pub content: String,
    pub due_date: Option<NaiveDate>,
}

pub struct PlanifyDropdownInit {
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub enum PlanifyDropdownMsg {}

#[derive(Debug)]
pub enum PlanifyDropdownCmd {
    Tasks {
        overdue: Vec<PlanifyTask>,
        today: Vec<PlanifyTask>,
        upcoming: Vec<PlanifyTask>,
    },
    ScaleChanged(f32),
}
