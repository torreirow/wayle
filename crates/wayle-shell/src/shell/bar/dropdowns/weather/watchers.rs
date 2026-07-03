use std::sync::Arc;

use relm4::ComponentSender;
use wayle_config::ConfigService;
use wayle_widgets::watch;

use super::{WeatherDropdown, messages::WeatherDropdownCmd};

pub(super) fn spawn(sender: &ComponentSender<WeatherDropdown>, config: &Arc<ConfigService>) {
    spawn_scale_watcher(sender, config);
}

fn spawn_scale_watcher(sender: &ComponentSender<WeatherDropdown>, config: &Arc<ConfigService>) {
    let scale = config.config().styling.scale.clone();

    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(WeatherDropdownCmd::ScaleChanged(scale.get().value()));
    });
}
