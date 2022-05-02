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

    loop {
        match client.framed_stream.next().await {
            None => {
                eprintln!("done.");
                std::process::exit(0);
            },
            Some(Ok(TrackerPacket::Update(update))) => {
                println!("got update: {:?}", update);
            },
            Some(Ok(TrackerPacket::Server(server))) => {
                println!("got server: {:?}", server);
            },
            Some(Ok(TrackerPacket::ResponseHeader)) => {
                println!("connected!");
            },
            Some(Ok(TrackerPacket::Complete)) => {
                println!("done!");
                std::process::exit(0);
            },
            Some(Err(err)) => {
                panic!("got something else: {err}");
                
            }
        }
    }
}
