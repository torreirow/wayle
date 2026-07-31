use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use relm4::ComponentSender;
use wayle_config::ConfigService;
use wayle_widgets::watch;

use super::{
    SolidtimeDropdown,
    messages::{RecentEntry, SolidtimeDropdownCmd},
};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const TICK_INTERVAL: Duration = Duration::from_secs(1);

pub(super) fn spawn(sender: &ComponentSender<SolidtimeDropdown>, config: &Arc<ConfigService>) {
    spawn_poll_watcher(sender);
    spawn_tick_watcher(sender);
    spawn_projects_watcher(sender);
    spawn_recent_watcher(sender);
    spawn_scale_watcher(sender, config);
}

fn spawn_poll_watcher(sender: &ComponentSender<SolidtimeDropdown>) {
    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                () = tokio::time::sleep(POLL_INTERVAL) => {
                    let result = query_current().await;
                    let _ = out.send(result);
                }
            }
        }
    });
}

fn spawn_tick_watcher(sender: &ComponentSender<SolidtimeDropdown>) {
    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                () = tokio::time::sleep(TICK_INTERVAL) => {
                    let _ = out.send(SolidtimeDropdownCmd::Tick);
                }
            }
        }
    });
}

fn spawn_projects_watcher(sender: &ComponentSender<SolidtimeDropdown>) {
    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        // Initial load
        let projects = fetch_projects().await;
        let _ = out.send(SolidtimeDropdownCmd::Projects(projects));

        shutdown_fut.await;
    });
}

fn spawn_recent_watcher(sender: &ComponentSender<SolidtimeDropdown>) {
    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        // Initial load
        let recent = fetch_recent().await;
        let _ = out.send(SolidtimeDropdownCmd::Recent(recent));

        shutdown_fut.await;
    });
}

fn spawn_scale_watcher(sender: &ComponentSender<SolidtimeDropdown>, config: &Arc<ConfigService>) {
    let scale = config.config().bar.scale.clone();

    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(SolidtimeDropdownCmd::ScaleChanged(scale.get().value()));
    });
}

async fn query_current() -> SolidtimeDropdownCmd {
    let output = tokio::process::Command::new("soltty")
        .args(["current", "--json"])
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => {
            return SolidtimeDropdownCmd::StatusUpdate {
                running: false,
                start: None,
                description: String::new(),
                project: None,
                connected: false,
            };
        }
    };

    let json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(_) => {
            return SolidtimeDropdownCmd::StatusUpdate {
                running: false,
                start: None,
                description: String::new(),
                project: None,
                connected: false,
            };
        }
    };

    let running = json.get("running").and_then(|v| v.as_bool()).unwrap_or(false);
    let start = json
        .get("start")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<DateTime<Utc>>().ok());
    let description = json
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let project = json
        .get("project")
        .and_then(|v| v.as_str())
        .map(String::from);

    SolidtimeDropdownCmd::StatusUpdate {
        running,
        start,
        description,
        project,
        connected: true,
    }
}

async fn fetch_projects() -> Vec<String> {
    let output = tokio::process::Command::new("soltty")
        .args(["list", "projects", "--json"])
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return vec![],
    };

    let json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    json.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

async fn fetch_recent() -> Vec<RecentEntry> {
    let output = tokio::process::Command::new("soltty")
        .args(["list", "--json", "--limit", "4"])
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return vec![],
    };

    let json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    json.as_array()
        .map(|arr| {
            arr.iter()
                .map(|e| RecentEntry {
                    description: e
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    project: e
                        .get("project")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    duration_secs: e
                        .get("duration")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) async fn run_soltty(args: Vec<String>) -> bool {
    tokio::process::Command::new("soltty")
        .args(args)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub(super) async fn query_current_cmd() -> SolidtimeDropdownCmd {
    query_current().await
}
