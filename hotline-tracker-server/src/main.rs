mod registration_listener;
mod server_registry;

use server_registry::ServerRegistry;

#[tokio::main]
async fn main() {
    let addr = format!("0.0.0.0:{}", registration_listener::REGISTRATION_LISTEN_PORT);
    let mut listener = registration_listener::RegistrationListener::listen(&addr).await.unwrap();

    let mut registry = ServerRegistry::new();

    while let Ok((addr, r)) = listener.next_registration().await {
        println!("got record: {}: {}", r.name, r.description);
        println!("  {}:{} [{}]", addr, r.port, r.id);

        registry.register(addr, r);

        eprintln!("{:?}", registry);
    }

    println!("meh.");

}
