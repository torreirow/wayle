use std::sync::Arc;

use relm4::ComponentSender;
use wayle_weather::{WeatherService, WeatherStatus};
use wayle_widgets::watch;

use super::{
    LocationPanel,
    messages::{LocationPanelCmd, PageState},
};

pub(super) fn spawn(sender: &ComponentSender<LocationPanel>, weather: &Arc<WeatherService>) {
    let status_prop = weather.status.clone();

    watch!(sender, [status_prop.watch()], |out| {
        let status = status_prop.get();
        let (page, error) = match &status {
            WeatherStatus::Loading => (PageState::Loading, None),
            WeatherStatus::Loaded => (PageState::Loaded, None),
            WeatherStatus::Error(kind) => (PageState::Error, Some(kind.clone())),
        };
        let _ = out.send(LocationPanelCmd::PageChanged { page, error });
    });
}
