#![type_length_limit="1546013"]

mod bootstrap;
#[allow(dead_code)]
mod constants;
mod data;
mod error;
mod manager;
mod network;
pub mod utils;
pub use error::Error;

use network::Server;

#[macro_use]
extern crate log;
#[macro_use]
extern crate ensicoin_serializer_derive;

use std::{io::prelude::*, str::FromStr};
#[cfg(feature = "cli-config")]
use structopt::StructOpt;

#[derive(Debug)]
enum LogLevel {
    Debug,
    Error,
    Trace,
    Info,
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower: &str = &s.to_ascii_lowercase();
        match lower {
            "debug" => Ok(Self::Debug),
            "error" => Ok(Self::Error),
            "trace" => Ok(Self::Trace),
            "info" => Ok(Self::Info),
            s => Err(format!("Unknown log level: {}", s)),
        }
    }
}

#[cfg(feature = "cli-config")]
#[derive(StructOpt)]
#[structopt(name = "arcd", about = "An ensicoin node in rust")]
struct Config {
    #[structopt(short, long, default_value = "info")]
    /// Sets the log level (can be "trace","debug", "info", "error")
    pub log: LogLevel,
    #[structopt(long)]
    /// Cleans all data from previous executions
    pub clean: bool,
    #[structopt(short, long)]
    /// Saves the parameters as defaults for the next execution
    pub save: bool,
    #[structopt(long)]
    /// Uses a saved config
    pub use_config: bool,
    #[structopt(flatten)]
    pub server_config: ServerConfig,
}

#[cfg_attr(feature = "cli-config", derive(StructOpt))]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    #[cfg_attr(
        feature = "cli-config",
        structopt(short = "c", long = "connections", default_value = "42")
    )]
    /// Sets the maximum number of connections
    pub max_connections: u64,
    #[cfg_attr(feature = "cli-config", structopt(long))]
    /// Changes the default directory
    pub data_dir: Option<std::path::PathBuf>,
    #[cfg_attr(feature = "cli-config", structopt(short, long, default_value = "4224"))]
    /// Port listening for connections
    pub port: u16,
    #[cfg(feature = "service_discover")]
    #[cfg_attr(feature = "cli-config", structopt(long))]
    /// URL of the service discovery service
    pub service_url: Option<String>,
    #[cfg(feature = "matrix_discover")]
    #[cfg_attr(feature = "cli-config", structopt(long))]
    /// RON credentials for matrix
    pub matrix_creds: Option<std::path::PathBuf>,
    #[cfg(feature = "grpc")]
    #[cfg_attr(feature = "cli-config", structopt(long, short, default_value = "4225"))]
    /// Port listening for gRPC requests
    pub grpc_port: u16,
    #[cfg(feature = "grpc")]
    #[cfg_attr(feature = "cli-config", structopt(long))]
    /// Restrict gRPC requests to localhost
    pub grpc_localhost: bool,
}

#[tokio::main]
async fn main() {
    // Setting up the data_dir
    #[cfg(feature = "cli-config")]
    let server_config: ServerConfig = {
        let mut config = Config::from_args();

        if config.server_config.data_dir.is_none() {
            let mut path = dirs::data_dir().unwrap();
            path.push(r"another-rust-coin");
            config.server_config.data_dir = Some(path);
        };
        let data_dir = config.server_config.data_dir.clone().unwrap();

        let log_level = match config.log {
            LogLevel::Debug => simplelog::LevelFilter::Debug,
            LogLevel::Info => simplelog::LevelFilter::Info,
            LogLevel::Error => simplelog::LevelFilter::Error,
            LogLevel::Trace => simplelog::LevelFilter::Trace,
        };
        simplelog::TermLogger::init(
            log_level,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
        )
        .unwrap();
        let mut should_bootstrap = match std::fs::create_dir_all(&data_dir) {
            Err(e) => e.kind() == std::io::ErrorKind::AlreadyExists,
            Ok(_) => false,
        };

        if let Some(s) = data_dir.to_str() {
            info!("Using {} as data directory", s);
        }

        let mut settings_path = data_dir.clone();
        settings_path.push("settings.ron");
        if !settings_path.exists() {
            should_bootstrap = true;
        }

        if config.clean {
            if let Err(e) = bootstrap::clean(data_dir.clone()) {
                eprintln!("Could not clean directory: {}", e);
            }
            should_bootstrap = true;
        };
        if should_bootstrap {
            if let Err(e) = bootstrap::bootstrap(&data_dir) {
                error!("Could not bootstrap: {}", e);
                return;
            }
        }

        let mut settings_file = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(settings_path)
        {
            Ok(f) => f,
            Err(e) => {
                warn!("Settings file could not be opened: {}", e);
                return;
            }
        };
        if config.save {
            let mut pretty = ron::ser::PrettyConfig::default();
            pretty.depth_limit = 4;
            pretty.separate_tuple_members = true;
            let config_string = match ron::ser::to_string_pretty(&config.server_config, pretty) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Could not serialize config: {}", e);
                    return;
                }
            };
            if let Err(e) = settings_file.write(config_string.as_bytes()) {
                warn!("Could not write config file: {}", e)
            }
        };
        if config.use_config {
            match ron::de::from_reader(settings_file) {
                Ok(s) => config.server_config = s,
                Err(e) => warn!("Could not use config file: {}", e),
            }
        }
        config.server_config
    };
    #[cfg(not(feature = "cli-config"))]
    let server_config: ServerConfig = {
        let mut data_dir = dirs::data_dir().unwrap();
        data_dir.push(r"another-rust-coin");

        let log_level = simplelog::LevelFilter::Info;
        simplelog::TermLogger::init(
            log_level,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
        )
        .unwrap();

        let mut should_bootstrap = match std::fs::create_dir_all(&data_dir) {
            Err(e) => e.kind() == std::io::ErrorKind::AlreadyExists,
            Ok(_) => false,
        };

        if let Some(s) = data_dir.to_str() {
            info!("Using {} as data directory", s);
        }

        let mut settings_path = data_dir.clone();
        settings_path.push("settings.ron");

        if !settings_path.exists() {
            should_bootstrap = true;
        }

        if should_bootstrap {
            if let Err(e) = bootstrap::bootstrap(&data_dir) {
                error!("Could not bootstrap: {}", e);
                return;
            }
        }

        let mut settings_file = match std::fs::File::open(settings_path) {
            Ok(f) => f,
            Err(e) => {
                warn!("Settings file could not be opened: {}", e);
                return;
            }
        };
        match ron::de::from_reader(settings_file) {
            Ok(s) => s,
            Err(e) => {
                error!("Could not use config file: {}", e);
                return;
            }
        }
    };

    if let Err(e) = Server::run(server_config).await {
        error!("Error running server: {}", e)
    };
}
