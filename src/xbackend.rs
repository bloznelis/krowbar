use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::properties::WmClass;
use x11rb::protocol::randr;
use x11rb::protocol::xproto::*;
use x11rb::protocol::xproto::{ConnectionExt, PropMode};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::config::KrowbarConfig;
use crate::config::Position;

pub struct X11Backend {
    conn: RustConnection,
    pub root_window: u32,
    pub atoms: AtomCollection,
    pub monitors: Vec<Monitor>,
}

#[derive(Debug)]
pub struct Monitor {
    pub name: String,
    x_offset: i16,
    y_offset: i16,
    width: u16,
    #[allow(warnings)]
    height: u16,
}

impl X11Backend {
    pub fn new() -> Result<X11Backend> {
        let (conn, screen_num) = x11rb::connect(None)?;

        let screen = conn.setup().roots[screen_num].clone();
        let atoms = AtomCollection::new(&conn)?.reply()?;

        let _ = randr::query_version(&conn, 1, 5)?.reply()?;
        let res = randr::get_screen_resources_current(&conn, screen.root)?.reply()?;

        let monitors: Vec<Monitor> = res
            .outputs
            .into_iter()
            .flat_map(|output| Self::output_to_monitor(output, &conn).transpose())
            .collect::<Result<Vec<Monitor>>>()?;

        Ok(X11Backend {
            conn,
            root_window: screen.root,
            atoms,
            monitors,
        })
    }

    fn output_to_monitor(output: u32, conn: &RustConnection) -> Result<Option<Monitor>> {
        let info = randr::get_output_info(&conn, output, 0)?.reply()?;

        if info.connection == randr::Connection::CONNECTED {
            if info.crtc != 0 {
                let crtc_info = randr::get_crtc_info(&conn, info.crtc, 0)?.reply()?;
                Ok(Some(Monitor {
                    name: String::from_utf8(info.name)?,
                    x_offset: crtc_info.x,
                    y_offset: crtc_info.y,
                    width: crtc_info.width,
                    height: crtc_info.height,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub fn setup(&self, x11_win: u32, monitor: &Monitor, cfg: KrowbarConfig) -> Result<()> {
        self._setup(x11_win, monitor, &cfg)?;
        self._setup(x11_win, monitor, &cfg)?; // Some X11 race conditions here.
        self._setup(x11_win, monitor, &cfg)?; // Doesn't hurt to do more.

        Ok(())
    }

    fn _setup(&self, x11_win: u32, monitor: &Monitor, cfg: &KrowbarConfig) -> Result<()> {
        self.set_as_dock(x11_win)?;
        self.reparent(x11_win, self.root_window)?;

        let (x, y) = match cfg.bar.position {
            Position::Top => (monitor.x_offset, monitor.y_offset),
            Position::Bottom => {
                let x = monitor.x_offset.into();
                let y = monitor.y_offset + monitor.height as i16 - cfg.bar.height as i16;

                (x, y)
            }
        };

        self.place_window_at(x11_win, x.into(), y.into())?;
        self.resize_window(x11_win, monitor.width.into(), cfg.bar.height.into())?;

        Ok(())
    }

    pub fn get_wm_class(&self, win: u32) -> Result<Option<String>> {
        let wm_class = WmClass::from_reply(
            self.conn
                .get_property(false, win, self.atoms.WM_CLASS, self.atoms.STRING, 0, 1024)?
                .reply()?,
        )?
        .and_then(|class| String::from_utf8(class.class().to_vec()).ok());

        Ok(wm_class)
    }

    pub fn reparent(&self, win: u32, new_parent_win: u32) -> Result<()> {
        self.conn.reparent_window(win, new_parent_win, 0, 0)?;
        self.conn.flush()?;

        Ok(())
    }

    pub fn set_as_dock(&self, win: u32) -> Result<()> {
        self.conn.change_property32(
            PropMode::REPLACE,
            win,
            self.atoms._NET_WM_WINDOW_TYPE,
            self.atoms.ATOM,
            &[self.atoms._NET_WM_WINDOW_TYPE_DOCK],
        )?;

        self.conn.flush()?;
        Ok(())
    }

    pub fn resize_window(&self, win: u32, width: u32, height: u32) -> Result<()> {
        self.conn.configure_window(
            win,
            &ConfigureWindowAux {
                width: Some(width as u32),
                height: Some(height as u32),
                ..ConfigureWindowAux::default()
            },
        )?;
        self.conn.flush()?;
        Ok(())
    }

    pub fn place_window_at(&self, win: u32, x: i32, y: i32) -> Result<()> {
        self.conn.configure_window(
            win,
            &ConfigureWindowAux {
                x: Some(x),
                y: Some(y),
                ..ConfigureWindowAux::default()
            },
        )?;
        self.conn.flush()?;
        Ok(())
    }
}

x11rb::atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_DOCK,
        _NET_WM_NAME,
        WM_NAME,
        UTF8_STRING,
        ATOM,
        WM_CLASS,
        STRING,
    }
}
