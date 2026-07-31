use relm4::prelude::*;

use super::{SolidtimeDropdown, messages::SolidtimeDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let config = services.config.clone();
        let init = SolidtimeDropdownInit { config };
        let controller = SolidtimeDropdown::builder().launch(init).detach();
        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
