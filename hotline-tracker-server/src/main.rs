mod registration_listener;
mod server_registry;
mod tracker_listener;
mod tracker_codec;

use registration_listener::RegistrationListener;
use server_registry::ServerRegistry;
use tracker_listener::TrackerListener;

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

    let mut listener = RegistrationListener::new(&opts.bind_address, RegistrationListener::REGISTRATION_LISTEN_PORT, tx).await?;

    let registry = Arc::new(Mutex::new(ServerRegistry::new()));

    let tcp_registry = registry.clone();

    let tracker_server = TrackerListener::new(&opts.bind_address, TrackerListener::TRACKER_LISTEN_PORT, tcp_registry).await?;

    tokio::spawn(async move {
        tracker_server.listen().await.unwrap();
    });

    tokio::spawn(async move {
        // start listening for registrations
        listener.listen().await.unwrap();
    });

    while let Some((addr, r)) = rx.recv().await {
        println!("got record: {}: {}", r.name, r.description);
        println!("  {}:{} [{}]", addr, r.port, r.id);

        if let Ok(mut registry) = registry.lock() {
            registry.register(addr, r);
            eprintln!("registered server.");
        }

        eprintln!("{:?}", registry);
    }

    Ok(())
}
