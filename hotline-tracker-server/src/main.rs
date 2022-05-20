mod registration_listener;
mod server_registry;
mod tracker_listener;
mod tracker_codec;

use registration_listener::RegistrationListener;
use server_registry::ServerRegistry;
use tracker_listener::TrackerListener;
use macroman_tools::MacRomanString;

use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::mpsc;

use clap::Parser;

// server registry listens on a channel
// registration listener sends registratoins through channel
// tracker server has the registry
// emit event stream?
// metrics?

#[derive(Parser, Debug)]
struct StartOptions {
    #[clap(long, default_value="0.0.0.0")]
    bind_address: String,

    #[clap(long)]
    password: Option<String>,
}

#[derive(Parser, Debug)]
enum Subcommand {
    Start(StartOptions),
}

#[derive(Parser, Debug)]
struct App {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[tokio::main]
async fn main() {
    let app = App::parse();

    match app.subcommand {
        Subcommand::Start(opts) => start(opts).await.unwrap(),
    }
}

async fn start(opts: StartOptions) -> Result<(), Box<dyn std::error::Error>> {
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
        if let Some(ref pw) = opts.password {
            let pw = MacRomanString::from(pw.as_str());
            if pw != r.password {
                eprintln!("Rejected record from {}@{addr}:{}", r.name, r.port);
                continue;
            }
        }

        // add to registry
        if let Ok(mut registry) = registry.lock() {
            println!("Accepted record: {}@{addr}:{}", r.name, r.port);
            registry.register(addr, r);
        }
    }

    Ok(())
}
