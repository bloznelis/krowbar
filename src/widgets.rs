use battery::Battery;
use gtk::prelude::ButtonExt;
use gtk::prelude::*;
use gtk4 as gtk;
use std::{sync::Arc, time::Duration};

use chrono::Local;
use gtk::{Button, Label};
use sysinfo::{Disks, Networks, System};
use xbackend::X11Backend;

use crate::{
    bar::{EMPTY_DESKTOP, FOCUSED_DESKTOP, NON_EMPTY_DESKTOP, URGENT_DESKTOP},
    bspwm::MonitorState,
    instruments::{self},
    xbackend::{self},
};

pub struct DesktopButtons {
    pub buttons: Vec<DesktopButton>,
}

impl DesktopButtons {
    pub fn new(monitor_state: &MonitorState) -> Self {
        DesktopButtons {
            buttons: monitor_state
                .desktops
                .iter()
                .map(|desktop| {
                    let css = if desktop.is_active {
                        vec![FOCUSED_DESKTOP]
                    } else if desktop.is_urgent {
                        vec![URGENT_DESKTOP]
                    } else if desktop.node_count > 0 {
                        vec![NON_EMPTY_DESKTOP]
                    } else {
                        vec![EMPTY_DESKTOP]
                    };

                    let button = Button::builder()
                        .label(&desktop.desktop_name)
                        .css_classes(css)
                        .build();

                    let desktop_id = desktop.desktop_id.to_string();

                    //TODO: Use bspc socket directly https://github.com/andreykaere/bspc-rs/issues/4
                    button.connect_clicked(move |_| {
                        let _ = std::process::Command::new("bspc")
                            .arg("desktop")
                            .arg("-f")
                            .arg(&desktop_id)
                            .output();
                    });

                    DesktopButton {
                        button,
                        desktop_id: desktop.desktop_id,
                    }
                })
                .collect(),
        }
    }
}

pub struct DesktopButton {
    pub button: Button,
    pub desktop_id: u32,
}

pub struct WinCount {
    pub label: Label,
}

impl WinCount {
    pub fn new(state: &MonitorState) -> Self {
        WinCount {
            label: Label::builder()
                .css_name("win-count")
                .label(state.node_count_label())
                .build(),
        }
    }
}

pub struct ActiveNode {
    pub label: Label,
}

impl ActiveNode {
    pub fn new(x11: Arc<X11Backend>, monitor_state: &MonitorState) -> Self {
        let name: String = monitor_state
            .desktops
            .iter()
            .flat_map(|desktop_state| desktop_state.active_node)
            .collect::<Vec<u32>>()
            .first()
            .and_then(|focused| x11.clone().get_wm_class(*focused).ok())
            .flatten()
            .unwrap_or(String::new());

        let label = Label::builder()
            .css_name("active-node-name")
            .label(name)
            .build();

        ActiveNode { label }
    }
}

pub struct Storage {
    pub button: gtk::Button,
}

impl Storage {
    pub fn new(disks: &mut Disks) -> Self {
        let usage = Self::get_used_percentage(disks);
        let button = Button::builder()
            .css_name("storage")
            .label(Self::format(usage))
            .build();

        let mut storage = Storage { button };
        storage.set_css(usage);
        storage
    }

    fn set_css(&mut self, usage: f32) {
        if usage > 95. {
            self.button.set_css_classes(&["storage-high"]);
        } else if usage > 85. {
            self.button.set_css_classes(&["storage-mid"]);
        } else {
            self.button.set_css_classes(&["storage-low"]);
        }
    }

    pub fn refresh(&mut self, disks: &mut Disks) {
        let usage = Self::get_used_percentage(disks);
        let label = Self::format(usage);

        self.button.set_label(&label);
        self.set_css(usage);
    }

    fn get_used_percentage(disks: &mut Disks) -> f32 {
        disks.refresh();
        let mut total_space = 0;
        let mut total_available = 0;
        for disk in disks {
            total_space += disk.total_space();
            total_available += disk.available_space();
        }
        let total_used = total_space - total_available;

        total_used as f32 / total_space as f32 * 100.0
    }

    fn format(usage: f32) -> String {
        format!("DSK {:.0}%", usage)
    }
}

pub struct Cpu {
    pub button: gtk::Button,
}

impl Cpu {
    pub fn new(sys: &mut System) -> Self {
        let usage = Self::get_used_percentage(sys);
        let button = Button::builder()
            .css_name("cpu")
            .label(Self::format(usage))
            .build();

        let mut cpu = Cpu { button };
        cpu.set_css(usage);
        cpu
    }

    fn set_css(&mut self, usage: f32) {
        if usage > 90. {
            self.button.set_css_classes(&["cpu-high"]);
        } else if usage > 75. {
            self.button.set_css_classes(&["cpu-mid"]);
        } else {
            self.button.set_css_classes(&["cpu-low"]);
        }
    }

    pub fn refresh(&mut self, sys: &mut System) {
        let usage = &Self::get_used_percentage(sys);

        self.set_css(*usage);
        self.button.set_label(&Self::format(*usage))
    }

    fn get_used_percentage(sys: &mut System) -> f32 {
        sys.refresh_cpu_usage();
        sys.global_cpu_usage()
    }

    fn format(usage: f32) -> String {
        format!("CPU {:.0}%", usage)
    }
}

pub struct Mem {
    pub button: gtk::Button,
}

impl Mem {
    pub fn new(sys: &mut System) -> Self {
        sys.refresh_memory();
        let usage = Self::get_used_percentage(sys);
        let button = Button::builder()
            .css_name("mem")
            .label(Self::format(usage))
            .build();

        let mut mem = Mem { button };
        mem.set_css(usage);
        mem
    }

    fn set_css(&mut self, usage: f32) {
        if usage > 85. {
            self.button.set_css_classes(&["mem-high"]);
        } else if usage > 70. {
            self.button.set_css_classes(&["mem-mid"]);
        } else {
            self.button.set_css_classes(&["mem-low"]);
        }
    }

    pub fn refresh(&mut self, sys: &mut System) {
        let usage = Self::get_used_percentage(sys);
        let label = Self::format(usage);

        self.button.set_label(&label);
        self.set_css(usage);
    }

    fn get_used_percentage(sys: &mut System) -> f32 {
        sys.refresh_memory();
        sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0
    }

    fn format(usage: f32) -> String {
        format!("MEM {:.0}%", usage)
    }
}

pub struct Clock {
    short: bool,
    pub button: Button,
    pub cal: gtk::Calendar,
}

impl Clock {
    pub fn new() -> Self {
        let button = Button::builder()
            .css_name("clock")
            .label(format!("{}", Local::now().format("%H:%M")))
            .build();

        let cal = gtk::Calendar::builder().visible(false).build();

        Clock {
            short: true,
            button,
            cal,
        }
    }

    pub fn toggle_clock(&mut self) {
        self.short = !self.short;
        self.cal.set_visible(!self.short);
        self.refresh()
    }

    pub fn refresh(&self) {
        self.button.set_label(&Self::clock_now(self))
    }

    fn clock_now(&self) -> String {
        if self.short {
            format!("{}", Local::now().format("%H:%M"))
        } else {
            format!("{}", Local::now().format("%Y-%m-%d, %A @ %H:%M:%S"))
        }
    }
}

pub struct Batteries {
    pub buttons: Vec<Button>,
}

impl Batteries {
    pub fn new(bat_manager: &mut battery::Manager) -> anyhow::Result<Batteries> {
        let buttons = Self::fetch_batteries(bat_manager)?
            .into_iter()
            .enumerate()
            .map(|(idx, battery)| {
                let btn = Button::builder()
                    .label(Self::make_label(idx, &battery))
                    .css_name("battery")
                    .build();

                btn.set_css_classes(&[Self::choose_css_class(&battery)]);
                btn
            })
            .collect();

        Ok(Batteries { buttons })
    }

    pub fn refresh(&self, bat_manager: &mut battery::Manager) -> anyhow::Result<()> {
        let batteries = Self::fetch_batteries(bat_manager)?;

        for (idx, mut battery) in batteries.into_iter().enumerate() {
            let button = self
                .buttons
                .get(idx)
                .ok_or(anyhow::anyhow!("Failed to fetch bat button at index {idx}"))?;

            bat_manager.refresh(&mut battery)?;

            button.set_label(&Self::make_label(idx, &battery));
            button.set_css_classes(&[Self::choose_css_class(&battery)]);
        }

        Ok(())
    }

    fn fetch_batteries(bat_manager: &mut battery::Manager) -> anyhow::Result<Vec<Battery>> {
        let mut batteries: Vec<Battery> = bat_manager.batteries()?.flatten().collect();
        // XXX: best effort to maintain consistent ordering
        batteries.sort_by(|bat1, bat2| {
            let cmp1 = bat1.serial_number().or(bat1.model()).unwrap_or("");
            let cmp2 = bat2.serial_number().or(bat2.model()).unwrap_or("");

            cmp1.cmp(cmp2)
        });

        Ok(batteries)
    }

    fn make_label(idx: usize, battery: &Battery) -> String {
        let soc: f32 = battery.state_of_charge().into();
        let soc: f32 = soc * 100.0;

        let label = match battery.state() {
            battery::State::Charging => format!("{:.0}% {}", soc, Self::charging_label(battery)),
            battery::State::Discharging => {
                format!("{:.0}% {}", soc, Self::discharging_label(battery))
            }
            battery::State::Empty => String::from("EMPTY"),
            battery::State::Full => String::from("100%"),
            _ => format!("{:.0}%", soc),
        };

        format!("BAT{idx} {}", label)
    }

    fn choose_css_class(battery: &Battery) -> &str {
        let soc: f32 = battery.state_of_charge().into();
        let soc: f32 = soc * 100.0;

        if soc > 90. {
            "battery-high"
        } else if soc > 20. {
            "battery-mid"
        } else {
            "battery-low"
        }
    }

    fn charging_label(bat: &Battery) -> String {
        bat.time_to_full()
            .map(|time| Duration::from_secs(time.get::<battery::units::time::second>() as u64))
            .map(|dur| format!("(CHRG {})", Self::format_duration(dur)))
            .unwrap_or_default()
    }

    fn discharging_label(bat: &Battery) -> String {
        bat.time_to_empty()
            .map(|time| Duration::from_secs(time.get::<battery::units::time::second>() as u64))
            .map(|dur| format!("(DISCHRG {})", Self::format_duration(dur)))
            .unwrap_or_default()
    }

    fn format_duration(dur: Duration) -> String {
        let minutes = (dur.as_secs() / 60) % 60;
        let hours = (dur.as_secs() / 60) / 60;

        format!("{:0>2}:{:0>2}", hours, minutes)
    }
}

pub struct Network {
    pub label: gtk::Label,
}

impl Network {
    pub fn new(networks: &mut Networks) -> Self {
        let label = Self::fetch_networks_label(networks);
        let label = Label::builder().css_name("network").label(label).build();
        Network { label }
    }

    pub fn refresh(&mut self, networks: &mut Networks) {
        let networks_label = Self::fetch_networks_label(networks);
        self.label.set_label(&networks_label);
    }

    fn fetch_networks_label(networks: &mut Networks) -> String {
        networks.refresh();
        let mut labels: Vec<String> = networks
            .into_iter()
            // XXX: best effort to show only relevant network interfaces
            .filter(|(name, data)| data.total_received() > 0 && *name != "lo")
            .map(|(interface_name, data)| {
                format!(
                    "{interface_name}: ↓ {} / ↑ {}",
                    Self::format_size(data.received()),
                    Self::format_size(data.transmitted())
                )
            })
            .collect();

        labels.sort();
        labels.join(" ")
    }

    fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        let (value, unit) = if bytes >= GB {
            (bytes as f64 / GB as f64, "GB")
        } else if bytes >= MB {
            (bytes as f64 / MB as f64, "MB")
        } else if bytes >= KB {
            (bytes as f64 / KB as f64, "KB")
        } else {
            (bytes as f64, "B")
        };

        format!("{:.0} {}", value, unit)
    }
}

pub struct Volume {
    pub label: gtk::Label,
}

impl Volume {
    pub fn new() -> Self {
        let label = Self::fetch_volume_label();
        let label = Label::builder().css_name("volume").label(label).build();
        Volume { label }
    }

    pub fn refresh(&mut self) {
        let label = Self::fetch_volume_label();
        self.label.set_label(&label);
    }

    fn fetch_volume_label() -> String {
        instruments::alsa::get_volume_percents()
            .map(|vol| format!("VOL {:.0}%", vol))
            .unwrap_or(String::from("???"))
    }
}
