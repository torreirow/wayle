use std::rc::Rc;

use relm4::prelude::*;
use wayle_widgets::prelude::BarSettings;

use super::{SolidtimeInit, SolidtimeModule};
use crate::shell::{
    bar::{
        dropdowns::DropdownRegistry,
        modules::registry::{ModuleFactory, ModuleInstance, dynamic_controller},
    },
    services::ShellServices,
};

pub(crate) struct Factory;

impl ModuleFactory for Factory {
    fn create(
        settings: &BarSettings,
        services: &ShellServices,
        dropdowns: &Rc<DropdownRegistry>,
        class: Option<String>,
    ) -> Option<ModuleInstance> {
        let init = SolidtimeInit {
            settings: settings.clone(),
            config: services.config.clone(),
            dropdowns: dropdowns.clone(),
        };
        let controller = dynamic_controller(SolidtimeModule::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}
