use std::{path::PathBuf, sync::Arc, time::Duration};

use relm4::ComponentSender;
use wayle_config::ConfigService;
use wayle_widgets::watch;

use super::{
    PlanifyDropdown,
    helpers::{self, resolve_db_path},
    messages::PlanifyDropdownCmd,
};

const POLL_INTERVAL: Duration = Duration::from_secs(60);

pub(super) fn spawn(sender: &ComponentSender<PlanifyDropdown>, config: &Arc<ConfigService>) {
    spawn_poll_timer(sender, config);
    spawn_scale_watcher(sender, config);
}

fn spawn_poll_timer(sender: &ComponentSender<PlanifyDropdown>, config: &Arc<ConfigService>) {
    let cfg = config.config();
    let db_path = resolve_db_path(&cfg.modules.planify.db_path.get());
    let project = cfg.modules.planify.project.get().clone();

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                () = tokio::time::sleep(POLL_INTERVAL) => {
                    let db = db_path.clone();
                    let proj = project.clone();
                    let tasks = tokio::task::spawn_blocking(move || {
                        helpers::load_tasks(&db, &proj)
                    })
                    .await
                    .unwrap_or_default();

                    let _ = out.send(PlanifyDropdownCmd::Tasks {
                        overdue: tasks.overdue,
                        today: tasks.today,
                        upcoming: tasks.upcoming,
                    });
                }
            }
        }
    });
}

fn spawn_scale_watcher(sender: &ComponentSender<PlanifyDropdown>, config: &Arc<ConfigService>) {
    let scale = config.config().bar.scale.clone();

    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(PlanifyDropdownCmd::ScaleChanged(scale.get().value()));
    });
}
