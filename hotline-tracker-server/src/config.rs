use serde::Deserialize;

use log::debug;

use std::path::PathBuf;
use std::fs;

pub const DEFAULT_CONFIG_FILENAME: &str = "tracker.toml";

// load from (precidence):
// cli argument
// TRACKER_CONFIG environment variable
// ~/.hotline/tracker.conf
// ~/.config/hotline/tracker.conf
// /etc/hotline/tracker.conf

// if no config is found, then use ~/.config/hotline/tracker.conf
// database relative path is relative to config

#[derive(Debug)]
pub struct Config {
    pub loaded_from: Option<String>,
    pub base_path: PathBuf,
    pub bind_address: String,
    pub require_password: bool,
    pub database: String,
}

#[derive(Deserialize)]
pub struct ParsedConfig {
    server: ParsedServerConfig
}

#[derive(Deserialize)]
pub struct ParsedServerConfig {
    #[serde(rename = "bind-address")]
    pub bind_address: Option<String>,
    #[serde(rename = "require-password")]
    pub require_password: Option<bool>,
    pub database: Option<String>,
}

/// attempt to locate the tracker.toml file which contains the tracker server configuration. This
/// functoin will return the path to the config file itself if it's found or if it was passed in
/// explicitly, even if it doesn't exist.
pub fn find_config() -> Option<String> {
    if let Ok(path) = std::env::var("TRACKER_CONFIG") {
        return Some(path);
    }

    if let Ok(home_path) = std::env::var("HOME") {
        let mut path = PathBuf::from(&home_path);
        path.push(format!(".hotline/{DEFAULT_CONFIG_FILENAME}"));

        if path.exists() {
            return Some(path.to_str().unwrap().into());
        }

        let mut path = PathBuf::from(&home_path);
        path.push(format!(".config/hotline/{DEFAULT_CONFIG_FILENAME}"));

        if path.exists() {
            return Some(path.to_str().unwrap().into());
        }
    }

    // lastly, system-level config at /etc
    let path = PathBuf::from(format!("/etc/hotline/{DEFAULT_CONFIG_FILENAME}"));
    if path.exists() {
        return Some(path.to_str().unwrap().into());
    }

    // if no config found, return none
    None
}

/// try to load the config from the provided location
/// this will return a Config with default values placed in it. some of these will be calculated so
/// they're not just straight-up defaults.
pub fn load(path: String) -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = PathBuf::from(&path);

    if ! config_path.exists() {
        debug!("{path}: Config does't exist. Using default config.");

        let base_path = config_path.parent().unwrap();
        let mut database = base_path.to_path_buf();
        database.push("tracker.sqlite3");


        // just return the default
        return Ok(Config {
            loaded_from: None,
            base_path: base_path.into(),
            bind_address: "0.0.0.0".into(),
            require_password: false,
            database: database.to_str().unwrap().into(),
        })
    }

    debug!("Using config: {path}");

    // load it
    let config_data = std::fs::read_to_string(&path)?;
    let parsed_config: ParsedConfig = toml::from_str(&config_data)?;

    let base_path = config_path.parent().unwrap();
    let server_config = parsed_config.server;
    let database = server_config.database
        .map(|db| {
            let database = PathBuf::from(&db);

            if database.is_absolute() {
                debug!("Using absolute database path.");
                // it's absolute, let's use this value
                db
            } else {
                debug!("Using relative database path.");

                // it's relative... so it should be relative to our base_path
                let mut database = base_path.to_path_buf();
                database.push(db);
                fs::canonicalize(&database).unwrap().to_str().unwrap().into()
            }
        })
        .unwrap_or_else(|| {
        let mut database = base_path.to_path_buf();
        database.push("tracker.sqlite3");
        database.to_str().unwrap().into()
    });

    let bind_address = server_config.bind_address.unwrap_or_else(|| "0.0.0.0".into());
    let require_password = server_config.require_password.unwrap_or(false);

    Ok(Config {
        loaded_from: Some(path),
        base_path: base_path.into(),
        bind_address,
        require_password,
        database,
    })
}
