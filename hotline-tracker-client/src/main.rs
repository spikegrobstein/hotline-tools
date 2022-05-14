use futures::StreamExt;

mod client;

use client::Client;
use client::TrackerPacket;

use clap::Parser;

#[derive(Parser, Debug)]
struct ListArgs {
    /// The tracker to list servers from
    tracker: String,
}

#[derive(Parser, Debug)]
struct RegisterArgs {
    /// The tracker to register to
    tracker: String,

    /// The name of your server; how it will appear in the tracker
    #[clap(short, long)]
    name: String,

    /// How your server will be described in the listing
    #[clap(short, long)]
    description: String,

    /// The port for your Hotline server
    #[clap(short, long, default_value="5500")]
    port: u16,

    #[clap(short, long, default_value="0")]
    user_count: u16,
}

#[derive(Parser, Debug)]
enum Subcommand {
    List(ListArgs),
    Register(RegisterArgs),
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let result = match args.subcommand {
        Subcommand::List(list_args) => {
            list_tracker(&list_args).await
        },
        Subcommand::Register(_register_args) => {
            unimplemented!("not yet implemented.");
        },
    };

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
        std::process::exit(1);
    }
}

async fn list_tracker(args: &ListArgs) -> Result<(), Box<dyn std::error::Error>> {
    // FIXME: this currently only works with default port
    let mut client = Client::connect(&args.tracker, 5498).await?;

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

    Ok(())
}
