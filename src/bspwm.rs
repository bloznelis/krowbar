use std::future::IntoFuture;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use bspc_rs::events::{
    self, DesktopEvent, Event, NodeEvent, NodeFlagInfo, NodeStateInfo, Subscription,
};
use bspc_rs::properties::{Flag, State, Switch};
use bspc_rs::query;
use bspc_rs::selectors::{DesktopSelector, MonitorSelector, NodeSelector};

use crate::bar::SystemEvent;

type DesktopId = u32;

#[derive(Debug, Clone)]
pub struct BspwmState {
    pub monitors: Vec<MonitorState>,
}

#[derive(Debug, Clone)]
pub struct MonitorState {
    pub monitor_id: u32,
    pub monitor_name: String,
    pub desktops: Vec<DesktopState>,
}

#[derive(Debug, Clone)]
pub struct DesktopState {
    pub desktop_id: DesktopId,
    pub desktop_name: String,
    pub node_count: usize,
    pub is_active: bool,
    pub is_urgent: bool,
    pub is_active_node_fullscreen: bool,
    pub active_node: Option<u32>,
}

impl BspwmState {
    pub fn new() -> anyhow::Result<BspwmState> {
        let output = std::process::Command::new("bspc")
            .arg("query")
            .arg("-M")
            .arg("--names")
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let monitor_names = stdout.split_whitespace().map(String::from);

        let monitors: Vec<MonitorState> = monitor_names
            .map(|desktop_name| MonitorState::new(desktop_name))
            .collect::<anyhow::Result<Vec<MonitorState>>>()?;

        Ok(BspwmState { monitors })
    }

    pub fn find_monitor(&self, monitor_name: &str) -> anyhow::Result<&MonitorState> {
        self.monitors
            .iter()
            .find(|monitor| monitor.monitor_name == monitor_name)
            .ok_or(anyhow!(
                "Failed to find X11 monitor {}, in BSPWM state",
                monitor_name
            ))
    }

    pub fn find_monitor_by_id(&mut self, monitor_id: u32) -> anyhow::Result<&mut MonitorState> {
        self.monitors
            .iter_mut()
            .find(|monitor| monitor.monitor_id == monitor_id)
            .ok_or(anyhow!(
                "Failed to find X11 monitor {}, in BSPWM state",
                monitor_id
            ))
    }

    pub fn update_all_desktop_window_count(&mut self) {
        for monitor in self.monitors.iter_mut() {
            for desktop in monitor.desktops.iter_mut() {
                desktop.update_node_count();
            }
        }
    }
}

impl MonitorState {
    pub fn new(monitor_name: String) -> anyhow::Result<MonitorState> {
        let monitor_id = query::query_monitors(
            false,
            None,
            Some(MonitorSelector(&monitor_name)),
            None,
            None,
        )?
        .first()
        .ok_or(anyhow!("Monitor ID for for {monitor_name} was not found"))
        .copied()?;

        let output = std::process::Command::new("bspc")
            .arg("query")
            .arg("-D")
            .arg("--monitor")
            .arg(&monitor_name)
            .arg("--names")
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let desktop_names = stdout.split_whitespace().map(String::from);

        let active_desktop = Self::find_active_desktop(&monitor_name);
        let desktops: Vec<DesktopState> = desktop_names
            .map(|desktop_name| DesktopState::new(desktop_name, active_desktop))
            .collect::<anyhow::Result<Vec<DesktopState>>>()?;

        Ok(MonitorState {
            monitor_id,
            monitor_name,
            desktops,
        })
    }

    pub fn find_active_desktop(monitor_name: &str) -> Option<DesktopId> {
        query::query_desktops(
            false,
            None,
            Some(MonitorSelector(&monitor_name)),
            Some(DesktopSelector(".active")),
            None,
        )
        .ok()
        .and_then(|focused| focused.first().copied())
    }

    pub fn update_focused_desktop(&mut self, focused_id: u32) {
        for desktop in self.desktops.iter_mut() {
            let is_focused = desktop.desktop_id == focused_id;
            desktop.is_active = is_focused;
        }
    }

    pub fn update_active_node(&mut self, desktop_id: u32, active_node_id: u32) {
        for desktop in self.desktops.iter_mut() {
            if desktop.desktop_id == desktop_id {
                desktop.active_node = Some(active_node_id);
                desktop.is_active_node_fullscreen =
                    Self::is_node_focused(active_node_id, desktop_id).unwrap_or(false)
            } else {
                desktop.active_node = None;
                desktop.is_active_node_fullscreen = false;
            }
        }
    }

    pub fn is_node_focused(node_id: u32, desktop_id: u32) -> anyhow::Result<bool> {
        let result = query::query_nodes(
            Some(NodeSelector(&node_id.to_string())),
            None,
            Some(DesktopSelector(&desktop_id.to_string())),
            Some(NodeSelector(".fullscreen")),
        )?;

        Ok(!result.is_empty())
    }

    pub fn focused_desktop_node_count(&self) -> Option<usize> {
        self.focused_desktop_state()
            .map(|desktop| desktop.node_count)
    }

    pub fn find_desktop(&self, desktop_id: u32) -> Option<&DesktopState> {
        self.desktops
            .iter()
            .find(|desktop| desktop.desktop_id == desktop_id)
    }

    pub fn find_desktop_mut(&mut self, desktop_id: u32) -> Option<&mut DesktopState> {
        self.desktops
            .iter_mut()
            .find(|desktop| desktop.desktop_id == desktop_id)
    }

    pub fn find_active_node(&self) -> Option<u32> {
        self.desktops
            .iter()
            .filter(|desktop| desktop.is_active)
            .find_map(|desktop| desktop.active_node)
    }

    pub fn focused_desktop_state(&self) -> Option<&DesktopState> {
        self.desktops.iter().find(|desktop| desktop.is_active)
    }

    pub fn node_count_label(&self) -> String {
        match self.focused_desktop_node_count() {
            Some(count) if count > 0 => format!("[{}]", count),
            _ => String::new(),
        }
    }
}

impl DesktopState {
    pub fn new(
        desktop_name: String,
        active_desktop_id: Option<DesktopId>,
    ) -> anyhow::Result<DesktopState> {
        let desktop_id = query::query_desktops(
            false,
            None,
            None,
            Some(DesktopSelector(&desktop_name)),
            None,
        )?
        .first()
        .copied()
        .ok_or(anyhow!("No desktop ID found"))?;

        let node_count = Self::count_nodes(&desktop_name);

        let active_node: Option<u32> = query::query_nodes(
            None,
            None,
            Some(DesktopSelector(&desktop_name)),
            Some(NodeSelector(".active")),
        )
        .ok()
        .and_then(|nodes| nodes.first().copied());

        Ok(DesktopState {
            desktop_id,
            desktop_name,
            node_count,
            is_active: Some(desktop_id) == active_desktop_id,
            is_urgent: false,
            is_active_node_fullscreen: false,
            active_node,
        })
    }

    pub fn set_fullscreen(&mut self, is_fullscreen: bool) {
        self.is_active_node_fullscreen = is_fullscreen
    }

    pub fn set_urgent(&mut self, is_urgent: bool) {
        self.is_urgent = is_urgent
    }

    fn update_node_count(&mut self) {
        self.node_count = Self::count_nodes(&self.desktop_name);
    }

    fn count_nodes(desktop_name: &str) -> usize {
        query::query_nodes(
            None,
            None,
            Some(DesktopSelector(&desktop_name)),
            Some(NodeSelector(".window.!hidden")),
        )
        .map(|nodes| nodes.len())
        .unwrap_or(0)
    }
}

pub async fn listen_to_bspwm(
    sender: async_broadcast::Sender<SystemEvent>,
    state: Arc<Mutex<BspwmState>>
) -> anyhow::Result<()> {
    let subscriptions = vec![
        Subscription::DesktopFocus,
        Subscription::NodeAdd,
        Subscription::NodeRemove,
        Subscription::NodeFocus,
        Subscription::NodeState,
        Subscription::NodeFlag,
        Subscription::NodeSwap,
        Subscription::NodeTransfer,
    ];
    let mut subscriber = events::subscribe(false, None, &subscriptions)?;

    for event in subscriber.events() {
        //log::info!("event {:?}", event);
        let mut state = state.lock().expect("state mutex");
        match event? {
            Event::DesktopEvent(event) => match event {
                DesktopEvent::DesktopFocus(focus_info) => {
                    let updated_monitor = state.find_monitor_by_id(focus_info.monitor_id)?;
                    updated_monitor.update_focused_desktop(focus_info.desktop_id);

                    let ret = sender.broadcast_direct(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone())).await?;
                    drop(ret)
                }
                _ => {}
            },
            //Event::NodeEvent(NodeEvent::NodeFlag(NodeFlagInfo {
            //    monitor_id,
            //    desktop_id,
            //    flag: Flag::Urgent,
            //    switch,
            //    ..
            //})) => {
            //    let updated_monitor = state.find_monitor_by_id(monitor_id)?;
            //    let updated_desktop = updated_monitor.find_desktop_mut(desktop_id);
            //
            //    if let Some(desktop) = updated_desktop {
            //        match switch {
            //            Switch::On => desktop.set_urgent(true),
            //            Switch::Off => desktop.set_urgent(false),
            //        }
            //    }
            //
            //    let _ = sender
            //        .broadcast(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone()))
            //        .await?;
            //}
            //Event::NodeEvent(NodeEvent::NodeState(NodeStateInfo {
            //    monitor_id,
            //    desktop_id,
            //    state: State::Fullscreen,
            //    switch,
            //    ..
            //})) => {
            //    let updated_monitor = state.find_monitor_by_id(monitor_id)?;
            //    let updated_desktop = updated_monitor.find_desktop_mut(desktop_id);
            //    if let Some(desktop) = updated_desktop {
            //        match switch {
            //            Switch::On => desktop.set_fullscreen(true),
            //            Switch::Off => desktop.set_fullscreen(false),
            //        }
            //    }
            //
            //    let _ = sender
            //        .broadcast(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone()))
            //        .await?;
            //}
            //Event::NodeEvent(NodeEvent::NodeFocus(node_focus_info)) => {
            //    let updated_monitor = state.find_monitor_by_id(node_focus_info.monitor_id)?;
            //    updated_monitor
            //        .update_active_node(node_focus_info.desktop_id, node_focus_info.node_id);
            //
            //    let _ = sender
            //        .broadcast(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone()))
            //        .await?;
            //}
            //Event::NodeEvent(NodeEvent::NodeAdd(node_add_info)) => {
            //    state.update_all_desktop_window_count();
            //    let updated_monitor = state.find_monitor_by_id(node_add_info.monitor_id)?;
            //
            //    let _ = sender
            //        .broadcast(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone()))
            //        .await?;
            //}
            //Event::NodeEvent(NodeEvent::NodeRemove(node_remove_info)) => {
            //    state.update_all_desktop_window_count();
            //    let updated_monitor = state.find_monitor_by_id(node_remove_info.monitor_id)?;
            //
            //    let _ = sender
            //        .broadcast(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone()))
            //        .await?;
            //}
            //Event::NodeEvent(NodeEvent::NodeSwap(node_swap_info)) => {
            //    state.update_all_desktop_window_count();
            //    let updated_monitor = state.find_monitor_by_id(node_swap_info.dst_monitor_id)?;
            //
            //    let _ = sender
            //        .broadcast(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone()))
            //        .await?;
            //
            //    let updated_monitor = state.find_monitor_by_id(node_swap_info.src_monitor_id)?;
            //
            //    let _ = sender
            //        .broadcast(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone()))
            //        .await?;
            //}
            //Event::NodeEvent(NodeEvent::NodeTransfer(node_transfer_info)) => {
            //    state.update_all_desktop_window_count();
            //    let updated_monitor =
            //        state.find_monitor_by_id(node_transfer_info.dst_monitor_id)?;
            //
            //    let _ = sender
            //        .broadcast(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone()))
            //        .await?;
            //
            //    let updated_monitor =
            //        state.find_monitor_by_id(node_transfer_info.src_monitor_id)?;
            //
            //    let _ = sender
            //        .broadcast(SystemEvent::DesktopStateUpdateNew(updated_monitor.clone()))
            //        .await?;
            //}
            other => log::info!("unknown event {:?}", other),
        }
    }
    Ok(())
}
