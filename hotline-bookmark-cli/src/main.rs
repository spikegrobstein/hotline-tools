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
    /// The path to the bookmark file to operate on
    file: String,

    /// Output the bookmark file as JSON for external parsing
    #[clap(short, long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct CreateArgs {
    /// The path to the bookmark file we will create
    file: String,

    /// The address of the server for the bookmark
    address: String,

    /// Username to use to log in to the server
    #[clap(short, long, default_value="")]
    username: String,

    /// Password to use to log in to the server
    #[clap(short, long, default_value="")]
    password: String,
}

#[derive(Parser, Debug)]
enum Subcommand {
    /// Print the contents of an existing bookmark file
    Print(PrintArgs),

    /// Create a new bookmark file
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
                    print_bookmark(&bookmark, print_args);
                },
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            }
        },
        Subcommand::Create(create_args) => {
            match create_bookmark(create_args) {
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                },
                _ => {},
            }
        },
    }
}

fn print_bookmark(bookmark: &Bookmark, args: PrintArgs) {
    if args.json {
        unimplemented!("no json yet");
    } else {
        println!("Address: {}", bookmark.address);
        println!("Username: {}", bookmark.username);
        println!("Password: {}", bookmark.password);
    }
}

fn create_bookmark(args: CreateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut b = Bookmark::new(args.address.clone());
    b.credentials(args.username, args.password);
    b.write_to_file(&args.file)?;

    eprintln!("Wrote bookmark: {}", args.file);

    Ok(())
}
