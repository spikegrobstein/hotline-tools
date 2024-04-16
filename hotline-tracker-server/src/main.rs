#[macro_use]
extern crate diesel;

use log::{debug, error, info, warn};

use diesel::prelude::*;

mod banlist;
mod password;
mod schema;
mod util;

mod config;
mod registration_listener;
mod server_registry;
mod tracker_codec;
mod tracker_listener;

use registration_listener::RegistrationListener;
use server_registry::ServerRegistry;
use tracker_listener::TrackerListener;

use banlist::Banlist;
use password::Password;

use config::Config;

use std::sync::Arc;
use std::sync::Mutex;

use std::fs;
use std::process;

use tokio::sync::mpsc;

use clap::Parser;

use env_logger::{Builder, Env};

// config
// require-password (boolean)
// database file path
// listen interface

// server registry listens on a channel
// registration listener sends registratoins through channel
// tracker server has the registry
// emit event stream?
// metrics?

// banlist -----------------------

#[derive(Parser, Debug)]
enum BanlistSubcommand {
    /// Add a server to the banlist by ipv4 address
    Add(BanlistAddOptions),

    /// Remove a server from the banlist
    Remove(BanlistRemoveOptions),

    /// List all servers in the banlist
    List(BanlistListOptions),
}

#[derive(Parser, Debug)]
struct BanlistOptions {
    #[clap(subcommand)]
    subcommand: BanlistSubcommand,
}

#[derive(Parser, Debug)]
struct BanlistAddOptions {
    /// ipv4 address of server to add to banlist
    address: String,

    /// Notes for this entry in the banlist (a freeform string)
    #[clap(default_value = "")]
    notes: String,
}

#[derive(Parser, Debug)]
struct BanlistRemoveOptions {
    /// the ipv4 address of the server to remove from the banlist
    address: String,
}

#[derive(Parser, Debug)]
struct BanlistListOptions {}

// /banlist ----------------------

// passwords ---------------------

#[derive(Parser, Debug)]
struct PasswordOptions {
    #[clap(subcommand)]
    subcommand: PasswordSubcommand,
}

#[derive(Parser, Debug)]
enum PasswordSubcommand {
    /// Add an authorizzed password for server registrations
    Add(PasswordAddOptions),
    /// Remove an authorized password
    Remove(PasswordRemoveOptions),

    /// List all authorized passwords
    List(PasswordListOptions),
}

#[derive(Parser, Debug)]
struct PasswordAddOptions {
    /// The required password for servers to use when registering with this tracker
    password: String,

    /// Notes about this password entry (a freeform string)
    #[clap(default_value = "")]
    notes: String,
}

#[derive(Parser, Debug)]
struct PasswordRemoveOptions {
    /// The password used for authorizing server registrations.
    password: String,
}

#[derive(Parser, Debug)]
struct PasswordListOptions {}

// /password ---------------------

#[derive(Parser, Debug)]
struct StartOptions {
    /// The IP address to bind the server to and listen for requests and server registrations.
    #[clap(long)]
    bind_address: Option<String>,

    /// A required password for servers to pass in order to register with this tracker.
    /// Must be MacRoman compatible.
    #[clap(long)]
    require_password: bool,

    /// Explicitly remove any password requirement from the registration server. This will override
    /// whatever is in the config file.
    #[clap(long)]
    no_require_password: bool,
}

#[derive(Parser, Debug)]
enum Subcommand {
    /// Start the tracker server
    Start(StartOptions),

    /// Add and remove servers from the banlist
    Banlist(BanlistOptions),

    /// Add and remove passwords to limit registrations
    Password(PasswordOptions),
}

#[derive(Parser, Debug)]
struct App {
    #[clap(subcommand)]
    subcommand: Subcommand,

    /// Path to config file
    #[clap(long, short)]
    config: Option<String>,

    /// Override the path to the database file. By default this is adjacent to the config file or
    /// the current working directory if no config file.
    #[clap(long)]
    database: Option<String>,
}

#[tokio::main]
async fn main() {
    let app = App::parse();

    let env = Env::default().filter_or("TRACKER_LOG_LEVEL", "info");
    Builder::from_env(env).init();

    // try to find the config
    // if it's set on the CLI, use that
    // otherwise fall back to config::find_config()
    // and if that doesn't find it, default to current directory
    let config_path = app.config.or_else(config::find_config).unwrap_or_else(|| {
        debug!("Using current directory for data/config.");
        format!("./{}", config::DEFAULT_CONFIG_FILENAME)
    });

    let mut config = config::load(config_path).unwrap();

    // override the DB path in the config with the CLI arg if present
    if let Some(db) = app.database {
        config.database = db;
    }

    // make sure that config base dir exists; this is where we'll write the DB file
    fs::create_dir_all(&config.base_path).unwrap();

    debug!("Config: {:#?}", config);

    let connection = open_db(&config.database);

    match app.subcommand {
        Subcommand::Start(opts) => handle_start(connection, opts, config).await.unwrap(),
        Subcommand::Banlist(opts) => handle_banlist(connection, opts).await.unwrap(),
        Subcommand::Password(opts) => handle_password(connection, opts).await.unwrap(),
    }
}

fn open_db(database: &str) -> SqliteConnection {
    info!("Using database: {database}");
    SqliteConnection::establish(database).unwrap()
}

async fn handle_start(
    db: SqliteConnection,
    opts: StartOptions,
    mut config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(bind_address) = opts.bind_address {
        config.bind_address = bind_address;
    }

    let passwordcount = Password::len(&db)?;

    // if the user passed --require-password on the CLI
    // then assign that in our config as well, overriding whatever is there.
    if opts.require_password {
        debug!("CLI sets a password requirement");
        config.require_password = opts.require_password;
    } else if opts.no_require_password {
        debug!("CLI removes any password requirement.");
        config.require_password = false;
    }

    if config.require_password && passwordcount == 0 {
        warn!("Password is required but no passwords in database. Use 'password add <password>' to add new passwords.");
    } else if !config.require_password && passwordcount > 0 {
        warn!(
            "Password is not required but there are {passwordcount} password(s) in the database."
        );
    }

    // print some info
    info!("bind address: {}", config.bind_address);
    info!("require_password: {}", config.require_password);

    let (tx, mut rx) = mpsc::channel(32);

    let registry = Arc::new(Mutex::new(ServerRegistry::new()));

    let mut registration_listener = RegistrationListener::new(
        &config.bind_address,
        RegistrationListener::REGISTRATION_LISTEN_PORT,
        tx,
    )
    .await?;
    let tracker_server = TrackerListener::new(
        &config.bind_address,
        TrackerListener::TRACKER_LISTEN_PORT,
        registry.clone(),
    )
    .await?;

    // listen for listing connections
    tokio::spawn(async move {
        match tracker_server.listen().await {
            Ok(_) => {
                info!("Tracker server completed. Exiting.");
                process::exit(0);
            }
            Err(e) => {
                error!("Tracker server failed: {:?}", e);
                process::exit(1);
            }
        }
    });

    // listen for registrations. these will come through on the rx, from above.
    tokio::spawn(async move {
        // start listening for registrations
        match registration_listener.listen().await {
            Ok(_) => {
                info!("Registration server completed. Exiting.");
                process::exit(0);
            }
            Err(e) => {
                error!("Registration server failed: {:?}", e);
                process::exit(1);
            }
        }
    });

    // get each new registration as they come in and handle it
    // if we require a password, then validate that the password is correct
    // reject incorrect passwords
    // otherwise add to the registry
    while let Some((addr, r)) = rx.recv().await {
        // validate credentials
        if config.require_password && !Password::is_authorized(&db, &r.password).unwrap() {
            warn!(
                "Rejected record [bad credentials]: {} @ {addr}:{}",
                r.name, r.port
            );
            continue;
        }

        // check if server is in ban list
        match Banlist::is_banned(&db, &addr) {
            Ok(true) => {
                warn!("Rejected record [banned]: {} @ {addr}:{}", r.name, r.port);
                continue;
            }
            Ok(false) => {}
            Err(err) => {
                // report the error... skip the entry... move on.
                error!("Failed to check entry: {err}");
                continue;
            }
        }

        // add to registry
        if let Ok(mut registry) = registry.lock() {
            info!("Accepted record: {} @ {addr}:{}", r.name, r.port);
            registry.register(addr, r);
        }
    }

    Ok(())
}

async fn handle_banlist(
    db: SqliteConnection,
    opts: BanlistOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    match opts.subcommand {
        BanlistSubcommand::Add(s_opts) => Banlist::add(&db, &s_opts.address, &s_opts.notes)
            .and_then(|_| {
                eprintln!("Added {} to banlist.", s_opts.address);
                Ok(())
            }),

        BanlistSubcommand::Remove(s_opts) => Banlist::remove(&db, &s_opts.address).and_then(|_| {
            eprintln!("Removed {} from banlist.", s_opts.address);
            Ok(())
        }),

        BanlistSubcommand::List(s_opts) => handle_banlist_list(&db, s_opts),
    }
}

fn handle_banlist_list(
    db: &SqliteConnection,
    _opts: BanlistListOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let banlist = Banlist::list(db)?;

    if banlist.is_empty() {
        eprintln!("No servers in banlist.");
        return Ok(());
    }

    // print out the list of banned servers.
    for b in banlist {
        println!("{} {}", b.address, b.notes);
    }

    Ok(())
}

async fn handle_password(
    db: SqliteConnection,
    opts: PasswordOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    match opts.subcommand {
        PasswordSubcommand::Add(s_opts) => Password::add(&db, &s_opts.password, &s_opts.notes)
            .and_then(|_| {
                eprintln!("Added password to the password list.");
                Ok(())
            }),

        PasswordSubcommand::Remove(s_opts) => {
            Password::remove(&db, &s_opts.password).and_then(|_| {
                eprintln!("Removed password from password list.");
                Ok(())
            })
        }

        PasswordSubcommand::List(s_opts) => handle_password_list(&db, s_opts),
    }
}

fn handle_password_list(
    db: &SqliteConnection,
    _opts: PasswordListOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let passwords = Password::list(db)?;

    if passwords.is_empty() {
        eprintln!("No passwords in the database.");
        return Ok(());
    }

    for p in passwords {
        println!("{} {}", p.password, p.notes);
    }

    Ok(())
}
