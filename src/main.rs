mod bar;
mod bspwm;
mod widgets;
mod xbackend;
mod config;

use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Copy, Clone, Debug, ValueEnum, Hash, Eq, PartialEq)]
pub enum Widget {
    Desktops,
    WinCount,
    FocusedName,
    Network,
    Cpu,
    Mem,
    Disk,
    Bat,
    Clock,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, help = "Enable debug logging")]
    debug: bool,
    #[arg(long, value_delimiter = ',', help = "Enabled widgets")]
    pub enabled_widgets: Option<Vec<Widget>>,
    #[arg(
        long,
        value_delimiter = ',',
        help = "Disabled widgets (takes precedence over --enabled-widgets)"
    )]
    pub disabled_widgets: Option<Vec<Widget>>,
    #[arg(
        long,
        help = "Disable automatic padding. Useful when you want to manage padding yourself."
    )]
    pub no_pad: bool,
    #[arg(
        short,
        long,
        help = "Path to config. Defaults to ~/.config/krowbar/config.toml"
    )]
    pub config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    if args.debug {
        setup_logging().expect("Failed to setup debug logging");
    }
    let config = config::read(&args).expect("Failed to read config");

    ExitCode::from(bar::run(args, config) as u8)
}

fn setup_logging() -> anyhow::Result<()> {
    #[allow(deprecated)] // XXX: Warning regarding Windows, we don't care
    let log_path = std::env::home_dir()
        .ok_or(anyhow!("Failed to get home dir"))?
        .join("krowbar.log");

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} {m}{n}")))
        .build(log_path)?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}
