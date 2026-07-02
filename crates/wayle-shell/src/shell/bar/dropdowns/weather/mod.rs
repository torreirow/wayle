mod daily_forecast;
mod factory;
mod hourly_forecast;
mod location_panel;
mod messages;
mod methods;
mod stats_grid;
mod sun_times;
mod watchers;
mod weather_header;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_widgets::prelude::*;

pub(super) use self::factory::Factory;
use self::{
    location_panel::{LocationPanel, LocationPanelInit, LocationPanelInput},
    messages::{WeatherDropdownCmd, WeatherDropdownInit, WeatherDropdownInput},
};
use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 395.0;
const BASE_HEIGHT: f32 = 695.0;

pub(crate) struct WeatherDropdown {
    config: Arc<ConfigService>,
    panels: Vec<Controller<LocationPanel>>,
    active_location: usize,
    tab_labels: Vec<String>,
    show_tabs: bool,
    tab_buttons: Vec<gtk::Button>,
    scaled_width: i32,
    scaled_height: i32,
}

#[relm4::component(pub(crate))]
impl Component for WeatherDropdown {
    type Init = WeatherDropdownInit;
    type Input = WeatherDropdownInput;
    type Output = ();
    type CommandOutput = WeatherDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "weather-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,
            #[watch]
            set_height_request: model.scaled_height,

            connect_closed[sender] => move |_| {
                sender.input(WeatherDropdownInput::SwitchLocation(0));
            },

            #[template]
            Dropdown {

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_icon_name: Some("ld-sun-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-weather-title"),
                    },
                    #[template_child]
                    actions {
                        #[template]
                        GhostIconButton {
                            set_icon_name: "tb-refresh-symbolic",
                            set_tooltip_text: Some(&t!("dropdown-weather-refresh")),
                            connect_clicked => WeatherDropdownInput::Retry,
                        },
                    },
                },

                #[template]
                DropdownContent {
                    set_vexpand: true,

                    #[name = "tab_bar"]
                    gtk::Box {
                        add_css_class: "location-tab-bar",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 4,
                        #[watch]
                        set_visible: model.show_tabs,
                    },

                    #[name = "panel_stack"]
                    gtk::Stack {
                        set_transition_type: gtk::StackTransitionType::Crossfade,
                        set_vhomogeneous: true,
                        set_hhomogeneous: true,
                        set_vexpand: true,
                        #[watch]
                        set_visible_child_name: &model.active_panel_name(),
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let show_tabs = init.weather.len() > 1;
        let scale = init.config.config().styling.scale.get().value();

        let panels: Vec<Controller<LocationPanel>> = init
            .weather
            .iter()
            .zip(init.tab_labels.iter())
            .zip(init.location_strs.iter())
            .map(|((svc, label), loc_str)| {
                LocationPanel::builder()
                    .launch(LocationPanelInit {
                        weather: svc.clone(),
                        config: init.config.clone(),
                        display_name: Some(label.clone()),
                        location_str: loc_str.clone(),
                    })
                    .detach()
            })
            .collect();

        watchers::spawn(&sender, &init.config);

        let mut tab_buttons: Vec<gtk::Button> = Vec::new();

        let mut model = Self {
            config: init.config,
            active_location: 0,
            tab_labels: init.tab_labels,
            show_tabs,
            tab_buttons: Vec::new(),
            panels,
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            scaled_height: scaled_dimension(BASE_HEIGHT, scale),
        };

        let widgets = view_output!();

        // Register panels in the stack and build tab buttons
        for (i, panel) in model.panels.iter().enumerate() {
            let child_name = format!("location-{i}");
            widgets.panel_stack.add_named(panel.widget(), Some(&child_name));

            if model.show_tabs {
                let label = model.tab_labels.get(i).cloned().unwrap_or_default();
                let btn = gtk::Button::builder()
                    .label(&label)
                    .css_classes(if i == 0 {
                        vec!["location-tab", "location-tab-active"]
                    } else {
                        vec!["location-tab"]
                    })
                    .build();

                let sender_clone = sender.clone();
                btn.connect_clicked(move |_| {
                    sender_clone.input(WeatherDropdownInput::SwitchLocation(i));
                });

                widgets.tab_bar.append(&btn);
                tab_buttons.push(btn);
            }
        }

        model.tab_buttons = tab_buttons;
        widgets.panel_stack.set_visible_child_name("location-0");

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            WeatherDropdownInput::Retry => {
                if let Some(panel) = self.panels.get(self.active_location) {
                    panel.emit(LocationPanelInput::Retry);
                }
            }
            WeatherDropdownInput::SwitchLocation(idx) => {
                if self.show_tabs {
                    if let Some(old_btn) = self.tab_buttons.get(self.active_location) {
                        old_btn.remove_css_class("location-tab-active");
                    }
                    if let Some(new_btn) = self.tab_buttons.get(idx) {
                        new_btn.add_css_class("location-tab-active");
                    }
                }
                self.active_location = idx;
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            WeatherDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
                self.scaled_height = scaled_dimension(BASE_HEIGHT, scale);
            }
        }
    }

}

