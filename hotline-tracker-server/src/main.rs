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

// server registry listens on a channel
// registration listener sends registratoins through channel
// tracker server has the registry
// emit event stream?
// metrics?

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let mut listener = RegistrationListener::new("0.0.0.0", RegistrationListener::REGISTRATION_LISTEN_PORT, tx).await.unwrap();

    let registry = Arc::new(Mutex::new(ServerRegistry::new()));

    let tcp_registry = registry.clone();

    tokio::spawn(async move {
        let tracker_server = TrackerListener::new("0.0.0.0", TrackerListener::TRACKER_LISTEN_PORT, tcp_registry).await.unwrap();
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

    println!("meh.");

}
