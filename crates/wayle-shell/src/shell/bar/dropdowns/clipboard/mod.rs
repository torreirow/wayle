mod factory;
mod helpers;
mod messages;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_widgets::prelude::*;
use wayle_widgets::watch;

pub(super) use self::factory::Factory;
use self::messages::{ClipboardDropdownCmd, ClipboardDropdownInit, ClipboardDropdownMsg, ClipboardEntry};
use crate::shell::bar::dropdowns::scaled_dimension;

const BASE_WIDTH: f32 = 360.0;

pub(crate) struct ClipboardDropdown {
    entries: Vec<ClipboardEntry>,
    scaled_width: i32,
    list_box: gtk::ListBox,
}

#[relm4::component(pub(crate))]
impl Component for ClipboardDropdown {
    type Init = ClipboardDropdownInit;
    type Input = ClipboardDropdownMsg;
    type Output = ();
    type CommandOutput = ClipboardDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "clipboard-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,

            connect_map[sender] => move |_| {
                sender.input(ClipboardDropdownMsg::Refresh);
            },

            #[template]
            Dropdown {
                set_overflow: gtk::Overflow::Hidden,

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        set_icon_name: Some("edit-copy-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: "Clipboard",
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
                        list_box -> gtk::ListBox {
                            add_css_class: "clipboard-list",
                            set_selection_mode: gtk::SelectionMode::None,

                            connect_row_activated[sender] => move |_, row| {
                                sender.input(ClipboardDropdownMsg::EntrySelected(row.index()));
                            },
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
        let scale = cfg.bar.scale.get().value();
        let scaled_width = scaled_dimension(BASE_WIDTH, scale);
        let list_box = gtk::ListBox::new();

        let model = ClipboardDropdown {
            entries: vec![],
            scaled_width,
            list_box: list_box.clone(),
        };

        let list_box = &model.list_box;
        let widgets = view_output!();

        spawn_scale_watcher(&sender, &init.config);

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: ClipboardDropdownMsg,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            ClipboardDropdownMsg::Refresh => {
                sender.command(|out, _shutdown| async move {
                    let entries = helpers::load_entries().await;
                    let _ = out.send(ClipboardDropdownCmd::Entries(entries));
                });
            }
            ClipboardDropdownMsg::EntrySelected(index) => {
                if let Some(entry) = self.entries.get(index as usize) {
                    let raw = entry.raw.clone();
                    sender.command(move |_out, _shutdown| async move {
                        helpers::copy_entry(&raw).await;
                    });
                }
                root.popdown();
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: ClipboardDropdownCmd,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ClipboardDropdownCmd::Entries(entries) => {
                self.entries = entries;
                populate_list(&self.list_box, &self.entries);
            }
            ClipboardDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
            }
        }
    }
}

fn populate_list(list_box: &gtk::ListBox, entries: &[ClipboardEntry]) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    if entries.is_empty() {
        let label = gtk::Label::new(Some("Geen clipboard history"));
        label.add_css_class("clipboard-empty-label");
        label.set_halign(gtk::Align::Center);
        let row = gtk::ListBoxRow::new();
        row.set_child(Some(&label));
        list_box.append(&row);
        return;
    }

    for entry in entries {
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        content.add_css_class("clipboard-entry-content");

        let label = gtk::Label::new(Some(&entry.preview));
        label.add_css_class("clipboard-entry-label");
        label.set_halign(gtk::Align::Start);
        label.set_hexpand(true);
        label.set_single_line_mode(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_max_width_chars(48);
        content.append(&label);

        let row = gtk::ListBoxRow::new();
        row.add_css_class("clipboard-entry-row");
        row.set_child(Some(&content));
        list_box.append(&row);
    }
}

fn spawn_scale_watcher(sender: &ComponentSender<ClipboardDropdown>, config: &Arc<ConfigService>) {
    let scale = config.config().bar.scale.clone();
    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(ClipboardDropdownCmd::ScaleChanged(scale.get().value()));
    });
}
