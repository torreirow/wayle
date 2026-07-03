use relm4::prelude::*;

use super::{WeatherDropdown, messages::WeatherDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let weather = services.weather.clone();
        let config = services.config.clone();

        let cfg = config.config();
        let weather_cfg = &cfg.modules.weather;

        let location_info: Vec<(String, Option<String>)> = {
            let locs = weather_cfg.locations.get();
            if !locs.is_empty() {
                locs.iter()
                    .map(|e| (e.location.clone(), e.name.clone()))
                    .collect()
            } else {
                vec![(
                    weather_cfg.location.get().clone(),
                    weather_cfg.location_name.get().clone(),
                )]
            }
        };

        let tab_labels: Vec<String> = location_info
            .iter()
            .map(|(loc, name)| name.clone().unwrap_or_else(|| loc.clone()))
            .collect();

        let location_strs: Vec<String> = location_info.into_iter().map(|(loc, _)| loc).collect();

        let init = WeatherDropdownInit {
            weather,
            config,
            tab_labels,
            location_strs,
        };
        let controller = WeatherDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
