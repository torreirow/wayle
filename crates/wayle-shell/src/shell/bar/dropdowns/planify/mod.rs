mod factory;
mod helpers;
mod messages;
mod watchers;

use std::sync::Arc;

use chrono::NaiveDate;
use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_widgets::prelude::*;

pub(super) use self::factory::Factory;
use self::{
    helpers::{PlanifyTasks, load_tasks, resolve_db_path},
    messages::{PlanifyDropdownCmd, PlanifyDropdownInit, PlanifyDropdownMsg, PlanifyTask},
};
use crate::{i18n::t, process, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 360.0;
const OPEN_CMD: &str = "uwsm app -- io.github.alainm23.planify";

pub(crate) struct PlanifyDropdown {
    badge_count: usize,
    project_name: String,
    db_path: std::path::PathBuf,
    scaled_width: i32,
    content_box: gtk::Box,
}

#[relm4::component(pub(crate))]
impl Component for PlanifyDropdown {
    type Init = PlanifyDropdownInit;
    type Input = PlanifyDropdownMsg;
    type Output = ();
    type CommandOutput = PlanifyDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "planify-dropdown"],
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
                        set_icon_name: Some("checkbox-checked-symbolic"),
                    },
                    #[template_child]
                    label {
                        #[watch]
                        set_label: &model.project_name,
                    },
                    #[template_child]
                    actions {
                        #[template]
                        GhostIconButton {
                            set_icon_name: "external-link-symbolic",
                            set_tooltip_text: Some(&t!("planify-dropdown-open")),
                            connect_clicked => move |_| {
                                process::run_if_set(OPEN_CMD);
                            },
                        },
                    },
                },

                #[template]
                DropdownContent {
                    gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_vscrollbar_policy: gtk::PolicyType::Automatic,
                        set_max_content_height: 480,
                        set_propagate_natural_height: true,

                        #[local_ref]
                        content_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            add_css_class: "planify-task-list",
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let cfg = init.config.config();
        let db_path = resolve_db_path(&cfg.modules.planify.db_path.get());
        let project_name = cfg.modules.planify.project.get().clone();
        let scale = cfg.bar.scale.get().value();
        let scaled_width = scaled_dimension(BASE_WIDTH, scale);

        let tasks = load_tasks(&db_path, &project_name);
        let badge_count = tasks.total_count();

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        populate_content(&content_box, &tasks);

        let model = PlanifyDropdown {
            badge_count,
            project_name,
            db_path,
            scaled_width,
            content_box: content_box.clone(),
        };

        let content_box = &model.content_box;
        let widgets = view_output!();

        watchers::spawn(&sender, &init.config);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: PlanifyDropdownMsg, _sender: ComponentSender<Self>, _root: &Self::Root) {}

    fn update_cmd(
        &mut self,
        message: PlanifyDropdownCmd,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            PlanifyDropdownCmd::Tasks { overdue, today, upcoming } => {
                self.badge_count = overdue.len() + today.len() + upcoming.len();
                let tasks = PlanifyTasks { overdue, today, upcoming };
                populate_content(&self.content_box, &tasks);
            }
            PlanifyDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
            }
        }
    }
}

fn populate_content(content_box: &gtk::Box, tasks: &PlanifyTasks) {
    while let Some(child) = content_box.first_child() {
        content_box.remove(&child);
    }

    if tasks.overdue.is_empty() && tasks.today.is_empty() && tasks.upcoming.is_empty() {
        let empty = gtk::Label::new(Some(&t!("planify-empty")));
        empty.add_css_class("planify-empty-label");
        content_box.append(&empty);
        return;
    }

    if !tasks.overdue.is_empty() {
        content_box.append(&make_section_header(&t!("planify-section-overdue")));
        for task in &tasks.overdue {
            let row = make_task_row(task);
            row.add_css_class("planify-task-overdue");
            content_box.append(&row);
        }
    }

    if !tasks.today.is_empty() {
        content_box.append(&make_section_header(&t!("planify-section-today")));
        for task in &tasks.today {
            content_box.append(&make_task_row(task));
        }
    }

    if !tasks.upcoming.is_empty() {
        content_box.append(&make_section_header(&t!("planify-section-upcoming")));
        for task in &tasks.upcoming {
            content_box.append(&make_task_row(task));
        }
    }
}

fn make_section_header(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("section-label");
    label.set_halign(gtk::Align::Start);
    label.set_xalign(0.0);
    label
}

fn make_task_row(task: &PlanifyTask) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    row.add_css_class("planify-task-row");

    let label = gtk::Label::new(Some(&task.content));
    label.set_halign(gtk::Align::Start);
    label.set_hexpand(true);
    label.set_xalign(0.0);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.set_max_width_chars(36);
    row.append(&label);

    if let Some(date) = task.due_date {
        let date_str = format_date(date);
        let date_label = gtk::Label::new(Some(&date_str));
        date_label.add_css_class("planify-task-date");
        row.append(&date_label);
    }

    row
}

fn format_date(date: NaiveDate) -> String {
    let today = chrono::Local::now().date_naive();
    let days = (date - today).num_days();
    match days {
        0 => String::from("today"),
        1 => String::from("tomorrow"),
        -1 => String::from("yesterday"),
        d if d < 0 => format!("{} days ago", -d),
        d if d <= 7 => date.format("%a").to_string(),
        _ => date.format("%d %b").to_string(),
    }
}
