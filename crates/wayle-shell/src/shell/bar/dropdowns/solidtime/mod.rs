mod factory;
mod messages;
mod watchers;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_widgets::prelude::*;

pub(super) use self::factory::Factory;
use self::messages::{
    RecentEntry, SolidtimeDropdownCmd, SolidtimeDropdownInit, SolidtimeDropdownMsg,
};
use crate::shell::bar::dropdowns::scaled_dimension;

const BASE_WIDTH: f32 = 340.0;

pub(crate) struct SolidtimeDropdown {
    running: bool,
    start: Option<DateTime<Utc>>,
    connected: bool,
    scaled_width: i32,
    elapsed_label: gtk::Label,
    status_label: gtk::Label,
    active_desc_label: gtk::Label,
    status_dot: gtk::Box,
    conn_dot: gtk::Box,
    start_stop_button: gtk::Button,
    project_model: gtk::StringList,
    recent_box: gtk::Box,
    desc_entry: gtk::Entry,
    project_dropdown: gtk::DropDown,
}

#[relm4::component(pub(crate))]
impl Component for SolidtimeDropdown {
    type Init = SolidtimeDropdownInit;
    type Input = SolidtimeDropdownMsg;
    type Output = ();
    type CommandOutput = SolidtimeDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "solidtime-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,

            #[template]
            Dropdown {
                set_overflow: gtk::Overflow::Hidden,

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        set_icon_name: Some("ld-zap-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: "Solidtime",
                    },
                    #[template_child]
                    actions {
                        #[local_ref]
                        conn_dot -> gtk::Box {
                            add_css_class: "solidtime-conn-dot",
                        },
                    },
                },

                #[template]
                DropdownContent {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,

                        // Status card
                        gtk::Box {
                            add_css_class: "solidtime-status",
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,

                            #[local_ref]
                            status_dot -> gtk::Box {
                                add_css_class: "solidtime-status-dot",
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 2,
                                set_hexpand: true,

                                #[local_ref]
                                status_label -> gtk::Label {
                                    add_css_class: "solidtime-status-label",
                                    set_halign: gtk::Align::Start,
                                    set_label: "Idle",
                                },

                                #[local_ref]
                                active_desc_label -> gtk::Label {
                                    add_css_class: "solidtime-status-desc",
                                    set_halign: gtk::Align::Start,
                                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                                    set_max_width_chars: 30,
                                    set_visible: false,
                                },

                                #[local_ref]
                                elapsed_label -> gtk::Label {
                                    add_css_class: "solidtime-elapsed",
                                    set_halign: gtk::Align::Start,
                                    set_label: "00:00:00",
                                },
                            },
                        },

                        gtk::Separator {
                            set_orientation: gtk::Orientation::Horizontal,
                        },

                        // Description entry
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 4,

                            gtk::Label {
                                add_css_class: "solidtime-field-label",
                                set_halign: gtk::Align::Start,
                                set_label: "Description",
                            },

                            #[local_ref]
                            desc_entry -> gtk::Entry {
                                set_placeholder_text: Some("What are you working on?"),
                                set_hexpand: true,
                                connect_activate[sender] => move |_| {
                                    sender.input(SolidtimeDropdownMsg::StartTimer);
                                },
                            },
                        },

                        // Project picker
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 4,

                            gtk::Label {
                                add_css_class: "solidtime-field-label",
                                set_halign: gtk::Align::Start,
                                set_label: "Project",
                            },

                            #[local_ref]
                            project_dropdown -> gtk::DropDown {
                                set_enable_search: true,
                                set_hexpand: true,
                            },
                        },

                        // Start/Stop button
                        #[local_ref]
                        start_stop_button -> gtk::Button {
                            add_css_class: "solidtime-start-stop",
                            set_label: "Start timer",
                            connect_clicked[sender] => move |btn| {
                                if btn.label().as_deref() == Some("Stop timer") {
                                    sender.input(SolidtimeDropdownMsg::StopTimer);
                                } else {
                                    sender.input(SolidtimeDropdownMsg::StartTimer);
                                }
                            },
                        },

                        gtk::Separator {
                            set_orientation: gtk::Orientation::Horizontal,
                        },

                        // Recent entries
                        gtk::Label {
                            add_css_class: "solidtime-field-label",
                            set_halign: gtk::Align::Start,
                            set_label: "Recent",
                        },

                        #[local_ref]
                        recent_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 0,
                        },
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
        let cfg = init.config.config();
        let scale = cfg.bar.scale.get().value();
        let scaled_width = scaled_dimension(BASE_WIDTH, scale);

        let project_model = gtk::StringList::new(&[]);
        let search_expr = gtk::PropertyExpression::new(
            gtk::StringObject::static_type(),
            gtk::Expression::NONE,
            "string",
        );
        let project_dropdown = gtk::DropDown::builder()
            .model(&project_model)
            .expression(&search_expr)
            .enable_search(true)
            .hexpand(true)
            .build();

        let elapsed_label = gtk::Label::new(Some("00:00:00"));
        let status_label = gtk::Label::new(Some("Idle"));
        let active_desc_label = gtk::Label::new(None);
        let status_dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let conn_dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let start_stop_button = gtk::Button::with_label("Start timer");
        let recent_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let desc_entry = gtk::Entry::new();

        conn_dot.add_css_class("solidtime-conn-dot");
        start_stop_button.set_css_classes(&["primary", "solidtime-start-stop"]);

        let model = SolidtimeDropdown {
            running: false,
            start: None,
            connected: false,
            scaled_width,
            elapsed_label: elapsed_label.clone(),
            status_label: status_label.clone(),
            active_desc_label: active_desc_label.clone(),
            status_dot: status_dot.clone(),
            conn_dot: conn_dot.clone(),
            start_stop_button: start_stop_button.clone(),
            project_model,
            recent_box: recent_box.clone(),
            desc_entry: desc_entry.clone(),
            project_dropdown: project_dropdown.clone(),
        };

        let elapsed_label = &model.elapsed_label;
        let status_label = &model.status_label;
        let active_desc_label = &model.active_desc_label;
        let status_dot = &model.status_dot;
        let conn_dot = &model.conn_dot;
        let start_stop_button = &model.start_stop_button;
        let recent_box = &model.recent_box;
        let desc_entry = &model.desc_entry;
        let project_dropdown = &model.project_dropdown;

        let widgets = view_output!();

        watchers::spawn(&sender, &init.config);

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: SolidtimeDropdownMsg,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            SolidtimeDropdownMsg::StartTimer => {
                let description = self.desc_entry.text().to_string();
                let project_idx = self.project_dropdown.selected();
                let project_name = if project_idx == u32::MAX {
                    None
                } else {
                    self.project_model
                        .string(project_idx)
                        .map(|s| s.to_string())
                };

                sender.command(move |out, _shutdown| async move {
                    let mut args = vec![String::from("start"), description.clone()];
                    if let Some(ref proj) = project_name {
                        args.push(String::from("-p"));
                        args.push(proj.clone());
                    }
                    args.push(String::from("-y"));

                    let ok = watchers::run_soltty(args).await;
                    if ok {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        let status = watchers::query_current_cmd().await;
                        let _ = out.send(status);
                    }
                });
            }
            SolidtimeDropdownMsg::StopTimer => {
                sender.command(move |out, _shutdown| async move {
                    let ok = watchers::run_soltty(vec![String::from("stop")]).await;
                    if ok {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        let status = watchers::query_current_cmd().await;
                        let _ = out.send(status);
                    }
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: SolidtimeDropdownCmd,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            SolidtimeDropdownCmd::StatusUpdate {
                running,
                start,
                description,
                project,
                connected,
            } => {
                self.running = running;
                self.start = start;
                self.connected = connected;

                // Update connection dot
                if connected {
                    self.conn_dot.remove_css_class("solidtime-conn-dot--bad");
                    self.conn_dot.add_css_class("solidtime-conn-dot--ok");
                } else {
                    self.conn_dot.remove_css_class("solidtime-conn-dot--ok");
                    self.conn_dot.add_css_class("solidtime-conn-dot--bad");
                }

                // Update status card
                if running {
                    self.status_label.set_label("Recording");
                    self.status_dot.add_css_class("solidtime-status-dot--running");
                    self.start_stop_button.set_label("Stop timer");
                    self.start_stop_button
                        .set_css_classes(&["danger", "solidtime-start-stop"]);

                    if !description.is_empty() {
                        let mut info = description.clone();
                        if let Some(proj) = &project {
                            info = format!("{} · {}", info, proj);
                        }
                        self.active_desc_label.set_label(&info);
                        self.active_desc_label.set_visible(true);
                    } else {
                        self.active_desc_label.set_visible(false);
                    }

                    // Update elapsed immediately
                    if let Some(s) = start {
                        let elapsed = Utc::now().signed_duration_since(s);
                        let total_secs = elapsed.num_seconds().max(0) as u64;
                        let h = total_secs / 3600;
                        let m = (total_secs % 3600) / 60;
                        let secs = total_secs % 60;
                        self.elapsed_label
                            .set_label(&format!("{:02}:{:02}:{:02}", h, m, secs));
                    }
                } else {
                    self.status_label.set_label("Idle");
                    self.status_dot
                        .remove_css_class("solidtime-status-dot--running");
                    self.start_stop_button.set_label("Start timer");
                    self.start_stop_button
                        .set_css_classes(&["primary", "solidtime-start-stop"]);
                    self.active_desc_label.set_visible(false);
                    self.elapsed_label.set_label("00:00:00");
                }
            }
            SolidtimeDropdownCmd::Tick => {
                if self.running {
                    if let Some(start) = self.start {
                        let elapsed = Utc::now().signed_duration_since(start);
                        let total_secs = elapsed.num_seconds().max(0) as u64;
                        let h = total_secs / 3600;
                        let m = (total_secs % 3600) / 60;
                        let s = total_secs % 60;
                        self.elapsed_label
                            .set_label(&format!("{:02}:{:02}:{:02}", h, m, s));
                    }
                }
            }
            SolidtimeDropdownCmd::Projects(projects) => {
                while self.project_model.n_items() > 0 {
                    self.project_model.remove(0);
                }
                for name in &projects {
                    self.project_model.append(name);
                }
            }
            SolidtimeDropdownCmd::Recent(entries) => {
                populate_recent(&self.recent_box, &entries);
            }
            SolidtimeDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
            }
        }
    }
}

fn populate_recent(recent_box: &gtk::Box, entries: &[RecentEntry]) {
    while let Some(child) = recent_box.first_child() {
        recent_box.remove(&child);
    }

    if entries.is_empty() {
        let empty = gtk::Label::new(Some("No recent entries"));
        empty.add_css_class("solidtime-recent-empty");
        empty.set_halign(gtk::Align::Start);
        recent_box.append(&empty);
        return;
    }

    for entry in entries {
        let row = make_recent_row(entry);
        recent_box.append(&row);
    }
}

fn make_recent_row(entry: &RecentEntry) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    row.add_css_class("solidtime-recent-row");

    let desc = gtk::Label::new(Some(&entry.description));
    desc.set_halign(gtk::Align::Start);
    desc.set_hexpand(true);
    desc.set_ellipsize(gtk::pango::EllipsizeMode::End);
    desc.set_max_width_chars(24);
    row.append(&desc);

    if let Some(proj) = &entry.project {
        let proj_label = gtk::Label::new(Some(proj));
        proj_label.add_css_class("solidtime-recent-project");
        row.append(&proj_label);
    }

    let total_secs = entry.duration_secs.max(0) as u64;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let dur_str = if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    };
    let dur_label = gtk::Label::new(Some(&dur_str));
    dur_label.add_css_class("solidtime-recent-duration");
    row.append(&dur_label);

    row
}
