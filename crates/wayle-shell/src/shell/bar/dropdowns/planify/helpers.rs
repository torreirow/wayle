use std::path::{Path, PathBuf};

use chrono::Local;
use rusqlite::{Connection, OpenFlags};
use tracing::warn;

use super::messages::PlanifyTask;

pub struct PlanifyTasks {
    pub overdue: Vec<PlanifyTask>,
    pub today: Vec<PlanifyTask>,
    pub upcoming: Vec<PlanifyTask>,
}

impl Default for PlanifyTasks {
    fn default() -> Self {
        Self {
            overdue: vec![],
            today: vec![],
            upcoming: vec![],
        }
    }
}

impl PlanifyTasks {
    pub fn total_count(&self) -> usize {
        self.overdue.len() + self.today.len() + self.upcoming.len()
    }
}

pub fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".local/share/io.github.alainm23.planify/database.db")
}

pub fn resolve_db_path(configured: &Option<String>) -> PathBuf {
    match configured {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => default_db_path(),
    }
}

pub fn load_tasks(db_path: &Path, project_name: &str) -> PlanifyTasks {
    match try_load_tasks(db_path, project_name) {
        Ok(tasks) => tasks,
        Err(e) => {
            warn!(error = %e, "planify: failed to load tasks");
            PlanifyTasks::default()
        }
    }
}

fn try_load_tasks(db_path: &Path, project_name: &str) -> rusqlite::Result<PlanifyTasks> {
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let project_id: String = conn.query_row(
        "SELECT id FROM Projects WHERE name = ?1 AND is_deleted = 0 LIMIT 1",
        [project_name],
        |row| row.get(0),
    )?;

    let today = Local::now().date_naive();

    let mut stmt = conn.prepare(
        "SELECT content, json_extract(due, '$.date') as due_date
         FROM Items
         WHERE project_id = ?1 AND checked = 0 AND is_deleted = 0",
    )?;

    let mut overdue = Vec::new();
    let mut today_tasks = Vec::new();
    let mut upcoming = Vec::new();

    let rows = stmt.query_map([&project_id], |row| {
        let content: String = row.get(0)?;
        let due_str: Option<String> = row.get(1)?;
        Ok((content, due_str))
    })?;

    for row in rows {
        let (content, due_str) = row?;
        let due_date = due_str
            .filter(|s| !s.is_empty())
            .and_then(|s| {
                let date_part = if s.len() >= 10 { &s[..10] } else { s.as_str() };
                chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()
            });

        let task = PlanifyTask { content, due_date };

        match due_date {
            None => today_tasks.push(task),
            Some(d) if d < today => overdue.push(task),
            Some(d) if d == today => today_tasks.push(task),
            _ => upcoming.push(task),
        }
    }

    overdue.sort_by_key(|t| t.due_date);
    upcoming.sort_by_key(|t| t.due_date);

    Ok(PlanifyTasks {
        overdue,
        today: today_tasks,
        upcoming,
    })
}
