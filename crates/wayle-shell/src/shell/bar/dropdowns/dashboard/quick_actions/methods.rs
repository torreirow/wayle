use relm4::ComponentSender;
use tracing::warn;
use wayle_power_profiles::types::profile::PowerProfile;

use super::{QuickActionsSection, messages::QuickActionsCmd};
use crate::i18n::t;

impl QuickActionsSection {
    pub(super) fn toggle_wifi(&self, sender: &ComponentSender<Self>) {
        let Some(network) = self.network.clone() else {
            return;
        };

        let target = !self.wifi_active;

        sender.oneshot_command(async move {
            if let Some(wifi) = network.wifi.get()
                && let Err(err) = wifi.set_enabled(target).await
            {
                warn!(error = %err, "wifi toggle failed");
            }
            QuickActionsCmd::WifiChanged(target)
        });
    }

    pub(super) fn toggle_bluetooth(&self, sender: &ComponentSender<Self>) {
        let Some(bluetooth) = self.bluetooth.get() else {
            return;
        };

        let target = !self.bluetooth_active;

        sender.oneshot_command(async move {
            let result = if target {
                bluetooth.enable().await
            } else {
                bluetooth.disable().await
            };
            if let Err(err) = result {
                warn!(error = %err, "bluetooth toggle failed");
            }
            QuickActionsCmd::BluetoothChanged(target)
        });
    }

    pub(super) fn toggle_airplane(&mut self, sender: &ComponentSender<Self>) {
        let target = !self.airplane_active;

        if target {
            self.pre_airplane_wifi = self.wifi_active;
            self.pre_airplane_bt = self.bluetooth_active;

            if self.wifi_active {
                self.toggle_wifi(sender);
            }
            if self.bluetooth_active {
                self.toggle_bluetooth(sender);
            }
        } else {
            if self.pre_airplane_wifi {
                self.toggle_wifi(sender);
            }
            if self.pre_airplane_bt {
                self.toggle_bluetooth(sender);
            }
        }

        self.airplane_active = target;
    }

    pub(super) fn toggle_dnd(&self, sender: &ComponentSender<Self>) {
        let Some(notification) = self.notification.clone() else {
            return;
        };

        let target = !self.dnd_active;

        sender.oneshot_command(async move {
            notification.set_dnd(target);
            QuickActionsCmd::DndChanged(target)
        });
    }

    pub(super) fn toggle_idle_inhibit(&self) {
        let state = self.idle_inhibit.state();
        if state.active.get() {
            state.disable();
        } else {
            state.enable(false);
        }
    }

    pub(super) fn cycle_power_profile(&self, sender: &ComponentSender<Self>) {
        let Some(power_profiles) = self.power_profiles.get() else {
            return;
        };

        let target = match self.active_profile {
            PowerProfile::PowerSaver if self.has_performance => PowerProfile::Performance,
            PowerProfile::PowerSaver => PowerProfile::Balanced,
            PowerProfile::Performance | PowerProfile::Unknown => PowerProfile::Balanced,
            PowerProfile::Balanced => PowerProfile::PowerSaver,
        };

        sender.oneshot_command(async move {
            if let Err(err) = power_profiles
                .power_profiles
                .set_active_profile(target)
                .await
            {
                warn!(error = %err, "power profile cycle failed");
            }
            QuickActionsCmd::PowerProfileChanged(target)
        });
    }

    pub(super) fn power_profile_icon(&self) -> &'static str {
        match self.active_profile {
            PowerProfile::PowerSaver => "ld-leaf-symbolic",
            PowerProfile::Performance => "ld-rocket-symbolic",
            PowerProfile::Balanced | PowerProfile::Unknown => "ld-scale-symbolic",
        }
    }

    pub(super) fn power_profile_label(&self) -> String {
        match self.active_profile {
            PowerProfile::PowerSaver => {
                t!("dropdown-dashboard-battery-profile-saver")
            }
            PowerProfile::Performance => {
                t!("dropdown-dashboard-battery-profile-performance")
            }
            PowerProfile::Balanced | PowerProfile::Unknown => {
                t!("dropdown-dashboard-battery-profile-balanced")
            }
        }
    }
}
