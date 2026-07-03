mod messages;
mod methods;
mod watchers;

use std::sync::Arc;

use gtk::{pango, prelude::*};
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_weather::{WeatherErrorKind, WeatherService};

pub(crate) use self::messages::{LocationPanelInit, LocationPanelInput};
use self::messages::{LocationPanelCmd, PageState};
use crate::i18n::t;
use super::{
    daily_forecast::{DailyForecast, DailyForecastInit},
    hourly_forecast::{HourlyForecast, HourlyForecastInit},
    stats_grid::{StatsGrid, StatsGridInit},
    sun_times::{SunTimes, SunTimesInit},
    weather_header::{WeatherHeader, WeatherHeaderInit},
};

pub(crate) struct LocationPanel {
    weather: Arc<WeatherService>,
    location_str: String,
    error_kind: Option<WeatherErrorKind>,
    page: PageState,

    weather_header: Controller<WeatherHeader>,
    stats_grid: Controller<StatsGrid>,
    hourly_forecast: Controller<HourlyForecast>,
    daily_forecast: Controller<DailyForecast>,
    sun_times: Controller<SunTimes>,
}

#[relm4::component(pub(crate))]
impl Component for LocationPanel {
    type Init = LocationPanelInit;
    type Input = LocationPanelInput;
    type Output = ();
    type CommandOutput = LocationPanelCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_vexpand: true,

            #[name = "page_stack"]
            gtk::Stack {
                set_transition_type: gtk::StackTransitionType::Crossfade,
                set_vhomogeneous: true,
                set_hhomogeneous: true,

                add_named[Some("loaded")] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand: true,

                    #[local_ref]
                    weather_header_widget -> gtk::Box {},

                    #[local_ref]
                    stats_grid_widget -> gtk::Box {},

                    #[local_ref]
                    hourly_forecast_widget -> gtk::Box {},

                    #[local_ref]
                    daily_forecast_widget -> gtk::Box {},

                    #[local_ref]
                    sun_times_widget -> gtk::Box {},
                },

                add_named[Some("loading")] = &gtk::Box {
                    add_css_class: "loading-weather",
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,
                    set_halign: gtk::Align::Fill,
                    set_vexpand: true,

                    gtk::Image {
                        add_css_class: "loading-icon",
                        set_icon_name: Some("ld-sun-symbolic"),
                    },

                    gtk::Label {
                        add_css_class: "loading-text",
                        set_label: &t!("dropdown-weather-loading"),
                        set_ellipsize: pango::EllipsizeMode::End,
                    },
                },

                add_named[Some("error")] = &gtk::Box {
                    add_css_class: "error-weather",
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,
                    set_halign: gtk::Align::Fill,
                    set_vexpand: true,

                    gtk::Image {
                        add_css_class: "error-icon",
                        set_icon_name: Some("ld-info-symbolic"),
                    },

                    gtk::Label {
                        add_css_class: "error-title",
                        set_label: &t!("dropdown-weather-error-title"),
                        set_ellipsize: pango::EllipsizeMode::End,
                    },

                    gtk::Label {
                        add_css_class: "error-text",
                        #[watch]
                        set_label: &model.error_description(),
                        set_ellipsize: pango::EllipsizeMode::End,
                    },

                    gtk::Button {
                        add_css_class: "weather-retry-btn",
                        set_halign: gtk::Align::Center,
                        set_cursor_from_name: Some("pointer"),
                        set_label: &t!("dropdown-weather-retry"),
                        connect_clicked => LocationPanelInput::Retry,
                    },
                },

                #[watch]
                set_visible_child_name: model.page.name(),
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let location_str = init.location_str.clone();

        let weather_header = WeatherHeader::builder()
            .launch(WeatherHeaderInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
                display_name_override: init.display_name.clone(),
            })
            .detach();

        let stats_grid = StatsGrid::builder()
            .launch(StatsGridInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
            })
            .detach();

        let hourly_forecast = HourlyForecast::builder()
            .launch(HourlyForecastInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
            })
            .detach();

        let daily_forecast = DailyForecast::builder()
            .launch(DailyForecastInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
            })
            .detach();

        let sun_times = SunTimes::builder()
            .launch(SunTimesInit {
                weather: init.weather.clone(),
                config: init.config.clone(),
            })
            .detach();

        let (page, error_kind) = match init.weather.status.get() {
            wayle_weather::WeatherStatus::Loading => (PageState::Loading, None),
            wayle_weather::WeatherStatus::Loaded => (PageState::Loaded, None),
            wayle_weather::WeatherStatus::Error(kind) => (PageState::Error, Some(kind)),
        };

        watchers::spawn(&sender, &init.weather);

        let model = Self {
            weather: init.weather,
            location_str,
            error_kind,
            page,
            weather_header,
            stats_grid,
            hourly_forecast,
            daily_forecast,
            sun_times,
        };

        let weather_header_widget = model.weather_header.widget();
        let stats_grid_widget = model.stats_grid.widget();
        let hourly_forecast_widget = model.hourly_forecast.widget();
        let daily_forecast_widget = model.daily_forecast.widget();
        let sun_times_widget = model.sun_times.widget();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            LocationPanelInput::Retry => self.trigger_refresh(),
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            LocationPanelCmd::PageChanged { page, error } => {
                self.page = page;
                self.error_kind = error;
            }
        }
    }
}

