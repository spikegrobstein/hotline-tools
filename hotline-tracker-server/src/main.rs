#[macro_use]
extern crate diesel;

use diesel::prelude::*;

mod schema;
mod banlist;
mod password;

mod registration_listener;
mod server_registry;
mod tracker_listener;
mod tracker_codec;

use registration_listener::RegistrationListener;
use server_registry::ServerRegistry;
use tracker_listener::TrackerListener;

use banlist::Banlist;
use password::Password;

use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::mpsc;

use clap::Parser;

// config
// banlist (array of addresses)
// passwords (array of passwords)
// require_password (boolean)

// server registry listens on a channel
// registration listener sends registratoins through channel
// tracker server has the registry
// emit event stream?
// metrics?

// banlist -----------------------

#[derive(Parser, Debug)]
enum BanlistSubcommand {
    Add(BanlistAddOptions),
    Remove(BanlistRemoveOptions),
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

    #[clap(default_value="")]
    notes: String,
}

#[derive(Parser, Debug)]
struct BanlistRemoveOptions {
    address: String,
}

#[derive(Parser, Debug)]
struct BanlistListOptions {
    
}

// /banlist ----------------------

#[derive(Parser, Debug)]
struct StartOptions {
    /// The IP address to bind the server to and listen for requests and server registrations.
    #[clap(long, default_value="0.0.0.0")]
    bind_address: String,

    /// A required password for servers to pass in order to register with this tracker.
    /// Must be MacRoman compatible.
    #[clap(long)]
    require_password: bool,
}

#[derive(Parser, Debug)]
enum Subcommand {
    /// Start the tracker server
    Start(StartOptions),
    Banlist(BanlistOptions),
}

#[derive(Parser, Debug)]
struct App {
    #[clap(subcommand)]
    subcommand: Subcommand,

    /// Path to the sqlite3 database
    #[clap(long, default_value="./tracker.sqlite3")]
    database: String,
}

#[tokio::main]
async fn main() {
    let app = App::parse();

    let connection = open_db(&app.database);

    match app.subcommand {
        Subcommand::Start(opts) =>
            handle_start(connection, opts).await.unwrap(),
        Subcommand::Banlist(opts) =>
            handle_banlist(connection, opts).await.unwrap(),
    }
}

fn open_db(database: &str) -> SqliteConnection {
    // todo: create the database if it doesn't exist
    SqliteConnection::establish(&database).unwrap()
}

async fn handle_start(db: SqliteConnection, opts: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(32);

    let registry = Arc::new(Mutex::new(ServerRegistry::new()));

    let mut registration_listener = RegistrationListener::new(&opts.bind_address, RegistrationListener::REGISTRATION_LISTEN_PORT, tx).await?;
    let tracker_server = TrackerListener::new(&opts.bind_address, TrackerListener::TRACKER_LISTEN_PORT, registry.clone()).await?;

    // listen for listing connections
    tokio::spawn(async move {
        tracker_server.listen().await.unwrap();
    });

    // listen for registrations. these will come through on the rx, from above.
    tokio::spawn(async move {
        // start listening for registrations
        registration_listener.listen().await.unwrap();
    });

    // get each new registration as they come in and handle it
    // if we require a password, then validate that the password is correct
    // reject incorrect passwords
    // otherwise add to the registry
    while let Some((addr, r)) = rx.recv().await {
        // validate credentials
        if opts.require_password {
            if ! Password::is_authorized(&db, &r.password).unwrap() {
                eprintln!("Rejected record [bad credentials]: {} @ {addr}:{}", r.name, r.port);
                continue;
            }
        }

        // check if server is in ban list
        match Banlist::is_banned(&db, &addr) {
            Ok(true) => {
                eprintln!("Rejected record [banned]: {} @ {addr}:{}", r.name, r.port);
                continue;
            },
            Ok(false) => {},
            Err(err) => {
                panic!("Failed to check entry: {err}");
            },
        }

        // add to registry
        if let Ok(mut registry) = registry.lock() {
            println!("Accepted record: {} @ {addr}:{}", r.name, r.port);
            registry.register(addr, r);
        }
    }

    Ok(())
}

async fn handle_banlist(db: SqliteConnection, opts: BanlistOptions) -> Result<(), Box<dyn std::error::Error>> {
    match opts.subcommand {
        BanlistSubcommand::Add(s_opts) =>
            Banlist::add(&db, s_opts.address, s_opts.notes),

        BanlistSubcommand::Remove(s_opts) =>
            Banlist::remove(&db, s_opts.address),

        BanlistSubcommand::List(s_opts) => 
            handle_banlist_list(&db, s_opts),
    }
}

fn handle_banlist_list(db: &SqliteConnection, opts: BanlistListOptions) -> Result<(), Box<dyn std::error::Error>> {
    let banlist = Banlist::list(db)?;

    if banlist.len() == 0 {
        eprintln!("No servers in banlist.");
        return Ok(())
    }

    // print out the list of banned servers.
    for b in banlist {
        println!("{} {}", b.address, b.notes);
    }

    Ok(())
}
