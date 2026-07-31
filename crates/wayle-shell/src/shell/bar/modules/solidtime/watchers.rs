use std::time::Duration;

use chrono::{DateTime, Utc};
use relm4::ComponentSender;

use super::{SolidtimeModule, messages::SolidtimeCmd};

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const TICK_INTERVAL: Duration = Duration::from_secs(1);

pub(super) fn spawn(sender: &ComponentSender<SolidtimeModule>) {
    spawn_poll_watcher(sender);
    spawn_tick_watcher(sender);
}

fn spawn_poll_watcher(sender: &ComponentSender<SolidtimeModule>) {
    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                () = tokio::time::sleep(POLL_INTERVAL) => {
                    let (running, start, description, project) = query_soltty_current().await;
                    let _ = out.send(SolidtimeCmd::StatusUpdate { running, start, description, project });
                }
            }
        }
    });
}

fn spawn_tick_watcher(sender: &ComponentSender<SolidtimeModule>) {
    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                () = tokio::time::sleep(TICK_INTERVAL) => {
                    let _ = out.send(SolidtimeCmd::Tick);
                }
            }
        }
    });
}

pub(super) async fn query_soltty_current() -> (bool, Option<DateTime<Utc>>, String, Option<String>) {
    let output = tokio::process::Command::new("soltty")
        .args(["current", "--json"])
        .output()
        .await;

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return (false, None, String::new(), None),
    };

    let json: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(_) => return (false, None, String::new(), None),
    };

    let running = json.get("running").and_then(|v| v.as_bool()).unwrap_or(false);
    if !running {
        return (false, None, String::new(), None);
    }

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

    (true, start, description, project)
}
