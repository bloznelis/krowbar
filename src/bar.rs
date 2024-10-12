use anyhow::anyhow;
use gtk4::{self as gtk};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};
use tinytemplate::TinyTemplate;

use gtk::{gdk, prelude::*, Application, ApplicationWindow};
use sysinfo::{Disks, Networks, System};
use xbackend::X11Backend;

use crate::{
    bspwm::{listen_to_bspwm, BspwmState, MonitorState},
    widgets::*,
    xbackend::{self, Monitor},
    Args, Widget, config::{CrowbarConfig, Position},
};

pub const FOCUSED_DESKTOP: &str = "focused-desktop";
pub const NON_EMPTY_DESKTOP: &str = "non-empty-desktop";
pub const URGENT_DESKTOP: &str = "urgent-desktop";
pub const EMPTY_DESKTOP: &str = "empty-desktop";

#[derive(Clone, Debug)]
enum BarEvent {
    ClockClick,
}

#[derive(Clone, Debug)]
pub enum SystemEvent {
    Tick,
    SlowTick,
    DesktopStateUpdateNew(MonitorState),
}

struct Bar {
    monitor_name: String,
    desktop_buttons: DesktopButtons,
    win_count: WinCount,
    active_node: ActiveNode,
    network: Network,
    cpu: Cpu,
    mem: Mem,
    storage: Storage,
    bat: Batteries,
    clock: Clock,
    bar_box: gtk::CenterBox,
}

pub struct Instruments {
    pub sys: System,
    pub networks: Networks,
    pub disks: Disks,
    pub bat_manager: battery::Manager,
}

impl Bar {
    fn new(
        monitor_state: &MonitorState,
        x11: Arc<X11Backend>,
        instruments: Arc<Mutex<Instruments>>,
        sender: async_channel::Sender<BarEvent>,
        monitor: &Monitor,
        args: &Args,
    ) -> Self {
        let bar_box = gtk::CenterBox::builder().build();

        let disabled_widgets: HashSet<Widget> = args
            .disabled_widgets
            .clone()
            .unwrap_or(vec![])
            .into_iter()
            .collect();
        let enabled_widgets: Vec<Widget> = args
            .enabled_widgets
            .clone()
            .unwrap_or(vec![
                Widget::Desktops,
                Widget::WinCount,
                Widget::FocusedName,
                Widget::Network,
                Widget::Cpu,
                Widget::Mem,
                Widget::Disk,
                Widget::Bat,
                Widget::Clock,
            ])
            .into_iter()
            .filter(|widget| !disabled_widgets.contains(widget))
            .collect();

        let box_left = gtk::Box::builder()
            .halign(gtk::Align::Start)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(2)
            .build();

        let box_center = gtk::Box::builder()
            .halign(gtk::Align::Center)
            .orientation(gtk::Orientation::Horizontal)
            .spacing(2)
            .build();

        let box_right = gtk::Box::builder()
            .halign(gtk::Align::End)
            .orientation(gtk::Orientation::Horizontal)
            .build();

        let Instruments {
            sys,
            networks,
            disks,
            bat_manager,
        } = &mut *instruments.lock().expect("instruments mutex");

        let desktop_buttons = DesktopButtons::new(&monitor_state);
        let win_count = WinCount::new(&monitor_state);
        let active_node = ActiveNode::new(x11.clone(), &monitor_state);
        let network = Network::new(networks);
        let cpu = Cpu::new(sys);
        let mem = Mem::new(sys);
        let storage = Storage::new(disks);
        let bat = Batteries::new(bat_manager).expect("Bat widget");
        let clock = Clock::new();

        //XXX: nasty hack. Avoids separators where they are not needed
        let mut sep_added = false;
        let mut add_sep = || {
            if sep_added {
                add_separator(&box_right);
            } else {
                sep_added = true;
            }
        };

        for enabled in enabled_widgets {
            match enabled {
                Widget::Desktops => {
                    for button in &desktop_buttons.buttons {
                        box_left.append(&button.button);
                    }
                }
                Widget::WinCount => {
                    box_left.append(&win_count.label);
                }
                Widget::FocusedName => {
                    box_left.append(&active_node.label);
                }
                Widget::Network => {
                    add_sep();
                    box_right.append(&network.label);
                }
                Widget::Cpu => {
                    add_sep();
                    box_right.append(&cpu.button);
                }
                Widget::Mem => {
                    add_sep();
                    box_right.append(&mem.button);
                }
                Widget::Disk => {
                    add_sep();
                    box_right.append(&storage.button);
                }
                Widget::Bat => {
                    for bat_btn in bat.buttons.iter() {
                        add_sep();
                        box_right.append(bat_btn);
                    }
                }
                Widget::Clock => {
                    box_right.append(&clock.button);
                }
            }
        }

        clock.button.connect_clicked(move |_| {
            let _ = sender.clone().send_blocking(BarEvent::ClockClick);
        });

        bar_box.set_start_widget(Some(&box_left));
        bar_box.set_center_widget(Some(&box_center));
        bar_box.set_end_widget(Some(&box_right));

        Bar {
            monitor_name: monitor.name.clone(),
            desktop_buttons,
            win_count,
            active_node,
            storage,
            network,
            cpu,
            mem,
            bat,
            clock,
            bar_box,
        }
    }
}

fn add_separator(gtk_box: &gtk::Box) {
    gtk_box.append(
        &gtk::Label::builder()
            .label("|")
            .css_name("separator")
            .build(),
    )
}

async fn react_to_updates(
    mut broadcast_receiver: async_broadcast::Receiver<SystemEvent>,
    channel_receiver: async_channel::Receiver<BarEvent>,
    mut bar: Bar,
    x11: Arc<X11Backend>,
    instruments: Arc<Mutex<Instruments>>,
    window: Arc<ApplicationWindow>,
) -> anyhow::Result<()> {
    loop {
        tokio::select! {
            system_event = broadcast_receiver.recv() => {
                match system_event? {
                    SystemEvent::SlowTick => {
                        let Instruments {
                            networks,
                            disks,
                            bat_manager,
                            ..
                        } = &mut *instruments.lock().expect("instruments mutex");

                        let _ = &bar.network.refresh(networks);
                        let _ = &bar.bat.refresh(bat_manager);
                        let _ = &bar.storage.refresh(disks);
                    }
                    SystemEvent::Tick => {
                        let Instruments {
                            sys,
                            ..
                        } = &mut *instruments.lock().expect("instruments mutex");

                        let _ = &bar.clock.refresh();
                        let _ = &bar.cpu.refresh(sys);
                        let _ = &bar.mem.refresh(sys);
                    }
                    SystemEvent::DesktopStateUpdateNew(monitor) if monitor.monitor_name == bar.monitor_name => {
                        let label = monitor.node_count_label();
                        bar.win_count.label.set_text(&label);

                        let active_node_name = monitor
                            .find_active_node()
                            .and_then(|win| x11.get_wm_class(win).ok())
                            .flatten()
                            .unwrap_or(String::new());

                        bar.active_node.label.set_text(&active_node_name);

                        for button in bar.desktop_buttons.buttons.iter() {
                            let desktop = monitor.find_desktop(button.desktop_id);
                            let has_nodes = desktop
                                .map(|desktop| desktop.node_count > 0)
                                .unwrap_or(false);
                            let is_urgent = desktop
                                .map(|desktop| desktop.is_urgent)
                                .unwrap_or(false);

                            if Some(button.desktop_id) == monitor.focused_desktop_state().map(|desktop| desktop.desktop_id) {
                                button.button.set_css_classes(&[FOCUSED_DESKTOP]);
                            } else if is_urgent {
                                button.button.set_css_classes(&[URGENT_DESKTOP]);
                            } else if has_nodes {
                                button.button.set_css_classes(&[NON_EMPTY_DESKTOP]);
                            } else {
                                button.button.set_css_classes(&[EMPTY_DESKTOP]);
                            }
                        }

                        if let Some(desktop) = monitor.focused_desktop_state() {
                            if desktop.is_active_node_fullscreen {
                                window.set_visible(false)
                            } else {
                                window.set_visible(true)
                            }
                        }
                    }
                    _ => log::debug!("ignored event")
                }
            }
            bar_event = channel_receiver.recv() => {
                match bar_event? {
                    BarEvent::ClockClick => {
                        let _ = &bar.clock.toggle_clock();
                    }
                }
            }
        }
    }
}

pub fn run(args: Args, cfg: CrowbarConfig) -> i32 {
    let application = Application::builder().application_id("c.row.bar").build();

    let css_cfg = cfg.clone();
    application.connect_startup(move |_| match attach_css(css_cfg.clone()) {
        Ok(_) => log::info!("css attached"),
        Err(err) => log::error!("failed while attaching css {err}"),
    });

    application.connect_activate(move |app| match app_configure(app, args.clone(), cfg.clone()) {
        Ok(_) => log::info!("bar configured"),
        Err(err) => log::error!("failed while conifguring app {err}"),
    });

    //XXX: have to pass empty array here, because default `.run` implicitly tries to parse args,
    //making it clash with clap.
    application.run_with_args::<&str>(&[]).value()
}

fn app_configure(app: &Application, args: Args, cfg: CrowbarConfig) -> anyhow::Result<()> {
    let x11 =
        Arc::new(X11Backend::new().map_err(|op| anyhow!("Failed to init X11 backend {:?}", op))?);

    let state = BspwmState::new().expect("pls");

    let (sender, receiver) = async_broadcast::broadcast::<SystemEvent>(32);

    let sys = System::new_all();
    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();
    let bat_manager = battery::Manager::new().expect("Bat manager");

    let instruments = Arc::new(Mutex::new(Instruments {
        sys,
        networks,
        disks,
        bat_manager,
    }));

    x11.monitors
        .iter()
        .map(|monitor| {
            init_bar_window(
                app,
                monitor,
                x11.clone(),
                instruments.clone(),
                receiver.clone(),
                state.find_monitor(&monitor.name)?,
                &args,
                &cfg,
            )
        })
        .collect::<anyhow::Result<()>>()?;

    let sender_cloned = sender.clone();
    let _ = tokio::spawn(async move {
        match listen_to_bspwm(sender_cloned, state).await {
            Ok(_) => log::info!("ok"),
            Err(err) => log::error!("failed while listening to bspwm {:?}", err),
        }
    });

    let sender_tick = sender.clone();
    let _ = tokio::spawn(async move {
        // Might want to reduce the sleep, to get faster clock updates.
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let result = sender_tick.broadcast(SystemEvent::Tick).await;
            match result {
                Ok(_) => {}
                Err(err) => log::error!("Failed to broadcast the tick! {:?}", err),
            }
        }
    });

    let _ = tokio::spawn(async move {
        // Might want to reduce the sleep, to get faster clock updates.
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let result = sender.broadcast(SystemEvent::SlowTick).await;
            match result {
                Ok(_) => {}
                Err(err) => log::error!("Failed to broadcast the slow tick! {:?}", err),
            }
        }
    });

    Ok(())
}

fn attach_css(cfg: CrowbarConfig) -> anyhow::Result<()> {
    let provider = gtk::CssProvider::new();

    let mut tt = TinyTemplate::new();
    tt.add_template(
        "theme-template",
        include_str!("../resources/theme-template.scss.tmpl"),
    )?;
    tt.add_template(
        "font-template",
        include_str!("../resources/font-template.scss.tmpl"),
    )?;
    let theme_css = tt.render("theme-template", &cfg.theme)?;
    let font_css = tt.render("font-template", &cfg.font)?;

    let mut sass = String::new();

    let base = include_str!("../resources/base.scss");
    sass.push_str(&font_css);
    sass.push_str(&theme_css);
    sass.push_str(base);

    let css = grass::from_string(sass, &grass::Options::default()).expect("Valid css expected");
    provider.load_from_data(&css);

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().ok_or(anyhow!("Failed to get default display"))?,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    Ok(())
}

fn init_bar_window(
    app: &Application,
    monitor: &Monitor,
    x11: Arc<X11Backend>,
    instruments: Arc<Mutex<Instruments>>,
    receiver: async_broadcast::Receiver<SystemEvent>,
    monitor_state: &MonitorState,
    args: &Args,
    cfg: &CrowbarConfig,
) -> anyhow::Result<()> {
    let window = ApplicationWindow::builder()
        .application(app)
        .focusable(false)
        .can_focus(false)
        .title("crowbar")
        .css_name("crowbar")
        .build();

    let (sender, receiver_bar_event) = async_channel::bounded::<BarEvent>(1);

    let bar = Bar::new(
        monitor_state,
        x11.clone(),
        instruments.clone(),
        sender.clone(),
        monitor,
        args,
    );

    window.set_child(Some(&bar.bar_box));

    let x11_cloned = x11.clone();
    let window_ref = Arc::new(window);
    let window_ref_cloned = window_ref.clone();
    gtk::glib::spawn_future_local(async move {
        let result = react_to_updates(
            receiver,
            receiver_bar_event,
            bar,
            x11_cloned,
            instruments.clone(),
            window_ref_cloned,
        )
        .await;

        match result {
            Ok(_) => log::info!("ok"),
            Err(err) => log::error!("failed while listening for updates bspwm {:?}", err),
        }
    });

    window_ref.set_visible(true);

    let x11_win = window_ref
        .surface()
        .ok_or(anyhow!("No surface on window!"))?
        .downcast::<gdk4_x11::X11Surface>()
        .map_err(|_| anyhow!("Failed to cast GTK surface to X11 surface"))?
        .xid() as u32; //check if we can safely cast here

    // This sucks, but I don't know how to prevent GTK created window being focused by BSPWM.
    std::process::Command::new("bspc")
        .arg("config")
        .arg("-n")
        .arg(x11_win.to_string())
        .arg("border_width")
        .arg("0")
        .output()
        .map_err(|op| anyhow!("Failed while setting border_width {}", op))?;

    if !args.no_pad {
        match cfg.bar.position {
            Position::Top => {
                std::process::Command::new("bspc")
                    .arg("config")
                    .arg("top_padding")
                    .arg(cfg.bar.height.to_string())
                    .output()
                    .map_err(|op| anyhow!("Failed while setting top padding {}", op))?;
            }
            Position::Bottom => {
                std::process::Command::new("bspc")
                    .arg("config")
                    .arg("bottom_padding")
                    .arg(cfg.bar.height.to_string())
                    .output()
                    .map_err(|op| anyhow!("Failed while setting bottom padding {}", op))?;
            }
        }
    }

    x11.clone()
        .setup(x11_win, monitor, cfg.clone())
        .map_err(|op| anyhow!("Failed to setup window via X11 {:?}", op))?;

    Ok(())
}
