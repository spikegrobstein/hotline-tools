use std::process;

use hotline_bookmark::bookmark::Bookmark;

use clap::Parser;

// usage:
// hlbm print <bookmark>
// hlbm create <bookmark> <address> --username <username> --password <password>
// --json for json printing
// use `-` for <address> if using json on STDIN (from output from tracker)
// ~/.config/hlbm/hooks.json -- for hooks for connecting?

#[derive(Parser, Debug)]
struct PrintArgs {
    file: String,

    #[clap(short, long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct CreateArgs {
    file: String,
    address: String,

    #[clap(short, long)]
    username: Option<String>,

    #[clap(short, long)]
    password: Option<String>,
}

#[derive(Parser, Debug)]
enum Subcommand {
    Print(PrintArgs),
    Create(CreateArgs),
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

fn main() {
    let args = Args::parse();

    match args.subcommand {
        Subcommand::Print(print_args) => {
            match Bookmark::from_file(&print_args.file) {
                Ok(bookmark) => {
                    print_bookmark(&bookmark, &print_args);
                },
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            }
        },
        Subcommand::Create(create_args) => {
            println!("creating...");
            // let mut b2 = Bookmark::new("24.30.100.120".into());
            // b2.credentials("star".into(), "guest".into());
            // b2.write_to_file("./new_bookmark.hlbm").unwrap();

            // let b = Bookmark::from_file("./new_bookmark.hlbm").unwrap();

            // println!("{:?}", b);
        },
    }
}

fn print_bookmark(bookmark: &Bookmark, args: &PrintArgs) {
    if args.json {
        unimplemented!("no json yet");
    } else {
        println!("Address: {}", bookmark.address);
        println!("Username: {}", bookmark.username);
        println!("Password: {}", bookmark.password);
    }
}
