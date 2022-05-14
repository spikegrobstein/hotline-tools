use tokio::net::UdpSocket;

use futures::StreamExt;

mod client;

use client::Client;
use client::TrackerPacket;

use hotline_tracker::RegistrationRecord;
use macroman_tools::MacRomanString;

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

    /// [Required] The name of your server; how it will appear in the tracker
    #[clap(short, long)]
    name: String,

    /// [Required] How your server will be described in the listing
    #[clap(short, long)]
    description: String,

    /// The port for your Hotline server
    #[clap(short, long, default_value="5500")]
    port: u16,

    /// [Required] Number of users currently connected to this server.
    #[clap(short, long, default_value="0")]
    user_count: u16,

    /// [Required] The unique ID for this server
    #[clap(short, long)]
    id: u32,
}

#[derive(Parser, Debug)]
enum Subcommand {
    /// List servers on a given tracker
    List(ListArgs),

    /// Manually register your server to a tracker
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
        Subcommand::Register(register_args) => {
            register(&register_args).await
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

async fn register(args: &RegisterArgs) -> Result<(), Box<dyn std::error::Error>> {
    let name = MacRomanString::from(args.name.as_str());
    let description = MacRomanString::from(args.description.as_str());

    // create the registration record so we have something to send.
    let r = RegistrationRecord {
        port: args.port,
        users_online: args.user_count,
        id: args.id,
        name,
        description,
        ..RegistrationRecord::default()
    };

    // fling out a UDP packet to the tracker server.
    let socket = UdpSocket::bind("0.0.0.0:2000").await?;
    eprintln!("Socket opened...");
    let buf = r.to_bytes();
    eprintln!("Sending packet to {}", args.tracker);
    let addr = format!("{}:5499", &args.tracker);
    socket.send_to(&buf, &addr).await?;

    Ok(())
}
