use futures::StreamExt;

mod client;

use client::Client;
use client::TrackerPacket;

// usage
// hltracker list <server>
// --json for json output

#[tokio::main]
async fn main() {
    let mut client = Client::connect("hltracker.com", 5498).await.unwrap();

    while let Some(packet) = client.framed_stream.next().await {
        match packet {
            Ok(TrackerPacket::Update(update)) => {
                println!("got update:");
                println!("  users_online:  {}", update.users_online);
                println!("  total_servers: {}", update.total_servers);
            },
            Ok(TrackerPacket::Server(server)) => {
                println!("{} [{}:{}]", server.name, server.address, server.port);
                println!("  {}", server.description);
            },
            Ok(TrackerPacket::ResponseHeader) => {
                println!("connected!");
            },
            Ok(TrackerPacket::Complete) => {
                break;
            },
            Err(err) => {
                panic!("got something else: {err}");
                
            }
        }
    }

    eprintln!("done.");
    std::process::exit(0);
}
