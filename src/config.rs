use serde::{Deserialize, Serialize};
use std::fs;
use toml;

use crate::Args;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct CrowbarConfig {
    pub theme: Theme,
    pub font: Font,
    pub bar: Bar,
}

impl Default for CrowbarConfig {
    fn default() -> Self {
        CrowbarConfig {
            theme: Theme::default(),
            font: Font::default(),
            bar: Bar::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct Theme {
    pub fg: String,
    pub fg_dim: String,
    pub fg_bright: String,
    pub ok: String,
    pub ok_dim: String,
    pub alert: String,
    pub alert_dim: String,
    pub warn: String,
    pub warn_dim: String,
    pub bg: String,
    pub bg_dim: String,
    pub bright: String,
    pub bright_dim: String,
    pub accent: String,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            fg: "#ebc17a".to_string(),
            fg_dim: "#8b7653".to_string(),
            fg_bright: "#ebc17a".to_string(),
            ok: "#909d63".to_string(),
            ok_dim: "#5e6547".to_string(),
            alert: "#bc5653".to_string(),
            alert_dim: "#74423f".to_string(),
            warn: "#bc5653".to_string(),
            warn_dim: "#74423f".to_string(),
            bg: "#1c1c1c".to_string(),
            bg_dim: "#232323".to_string(),
            bright: "#cacaca".to_string(),
            bright_dim: "#828282".to_string(),
            accent: "#bc5653".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct Font {
    pub font_family: String,
    pub font_size: String,
    pub font_weight: String,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            font_family: "Terminess Nerd Font".to_string(),
            font_size: "16px".to_string(),
            font_weight: "bold".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct Bar {
    pub height: u16,
    pub position: Position,
}

impl Default for Bar {
    fn default() -> Self {
        Bar {
            height: 10,
            position: Position::Bottom,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Position {
    Top,
    Bottom,
}

pub fn read(args: &Args) -> anyhow::Result<CrowbarConfig> {
    let path = args.config.clone().unwrap_or(
        #[allow(deprecated)] // XXX: Warning regarding Windows, we don't care now
        std::env::home_dir()
            .expect("Home dir not found")
            .join(".config/crowbar/config.toml"),
    );

    let cfg = if fs::exists(&path)? {
        let contents = fs::read_to_string(&path)?;
        let config: CrowbarConfig = toml::from_str(&contents)?;
        config
    } else {
        CrowbarConfig::default()
    };

    Ok(cfg)
}
