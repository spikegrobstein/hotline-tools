mod registration_listener;
mod server_registry;
mod tracker_listener;
mod tracker_codec;

use server_registry::ServerRegistry;
use tracker_listener::TrackerListener;

use std::sync::Arc;
use std::sync::Mutex;

#[tokio::main]
async fn main() {
    let addr = format!("0.0.0.0:{}", registration_listener::REGISTRATION_LISTEN_PORT);
    let mut listener = registration_listener::RegistrationListener::listen(&addr).await.unwrap();

    let registry = Arc::new(Mutex::new(ServerRegistry::new()));

    let tcp_registry = registry.clone();

    tokio::spawn(async move {
    let addr = format!("0.0.0.0:{}", tracker_listener::TRACKER_LISTEN_PORT);
        let tracker_listener = TrackerListener::listen(&addr, tcp_registry).await.unwrap();
    });

    while let Ok((addr, r)) = listener.next_registration().await {
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
