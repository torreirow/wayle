mod factory;
mod messages;
mod watchers;

use std::{rc::Rc, sync::Arc};

use chrono::{DateTime, Utc};
use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::{
    prelude::{
        BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput,
        BarButtonOutput,
    },
    utils::force_window_resize,
};

pub(crate) use self::{
    factory::Factory,
    messages::{SolidtimeCmd, SolidtimeInit, SolidtimeMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct SolidtimeModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
    running: bool,
    start: Option<DateTime<Utc>>,
}

#[relm4::component(pub(crate))]
impl Component for SolidtimeModule {
    type Init = SolidtimeInit;
    type Input = SolidtimeMsg;
    type Output = ();
    type CommandOutput = SolidtimeCmd;

    view! {
        gtk::Box {
            add_css_class: "solidtime",

            #[local_ref]
            bar_button -> gtk::MenuButton {},
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let solidtime_config = &config.modules.solidtime;

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: solidtime_config.icon_name.get().clone(),
                label: String::from("⏱"),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: solidtime_config.icon_color.clone(),
                    label_color: solidtime_config.label_color.clone(),
                    icon_background: solidtime_config.icon_bg_color.clone(),
                    button_background: solidtime_config.button_bg_color.clone(),
                    border_color: solidtime_config.border_color.clone(),
                    auto_icon_color: CssToken::Accent,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: solidtime_config.label_max_length.clone(),
                    show_icon: solidtime_config.icon_show.clone(),
                    show_label: solidtime_config.label_show.clone(),
                    show_border: solidtime_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => SolidtimeMsg::LeftClick,
                BarButtonOutput::RightClick => SolidtimeMsg::RightClick,
                BarButtonOutput::MiddleClick => SolidtimeMsg::MiddleClick,
                BarButtonOutput::ScrollUp => SolidtimeMsg::ScrollUp,
                BarButtonOutput::ScrollDown => SolidtimeMsg::ScrollDown,
            });

        watchers::spawn(&sender);

        let model = Self {
            bar_button,
            config: init.config,
            dropdowns: init.dropdowns,
            running: false,
            start: None,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let solidtime = &self.config.config().modules.solidtime;

        let action = match msg {
            SolidtimeMsg::LeftClick => solidtime.left_click.get(),
            SolidtimeMsg::RightClick => solidtime.right_click.get(),
            SolidtimeMsg::MiddleClick => solidtime.middle_click.get(),
            SolidtimeMsg::ScrollUp => solidtime.scroll_up.get(),
            SolidtimeMsg::ScrollDown => solidtime.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: SolidtimeCmd, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            SolidtimeCmd::StatusUpdate { running, start } => {
                self.running = running;
                self.start = start;

                if !running {
                    self.bar_button.emit(BarButtonInput::SetLabel(String::from("⏱")));
                    force_window_resize(root);
                }
            }
            SolidtimeCmd::Tick => {
                if self.running {
                    if let Some(start) = self.start {
                        let elapsed = Utc::now().signed_duration_since(start);
                        let total_secs = elapsed.num_seconds().max(0) as u64;
                        let h = total_secs / 3600;
                        let m = (total_secs % 3600) / 60;
                        let s = total_secs % 60;
                        let label = format!("⏱ {:02}:{:02}:{:02}", h, m, s);
                        self.bar_button.emit(BarButtonInput::SetLabel(label));
                        force_window_resize(root);
                    }
                }
            }
        }
    }
}
