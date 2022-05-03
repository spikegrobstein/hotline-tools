mod registration_listener;

#[tokio::main]
async fn main() {
    let addr = format!("127.0.0.1:{}", registration_listener::REGISTRATION_LISTEN_PORT);
    let mut listener = registration_listener::RegistrationListener::listen(&addr).await.unwrap();

    while let Ok((addr, r)) = listener.next_registration().await {
        println!("got record: {}: {}", r.name, r.description);
        println!("  {}:{} [{}]", addr, r.port, r.id);
    }

    println!("meh.");

}
