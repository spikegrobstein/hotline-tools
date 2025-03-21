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
    #[clap(short, long, default_value = "")]
    username: String,

    /// Password to use to log in to the server
    #[clap(short, long, default_value = "")]
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

    let result = match args.subcommand {
        Subcommand::Print(print_args) => print_bookmark(print_args),
        Subcommand::Create(create_args) => create_bookmark(create_args),
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn print_bookmark(args: PrintArgs) -> Result<(), Box<dyn std::error::Error>> {
    let bookmark = Bookmark::from_file(&args.file)?;

    if args.json {
        let json = serde_json::to_string(&bookmark)?;
        println!("{}", json);
    } else {
        println!("Address: {}", bookmark.address);
        println!("Username: {}", bookmark.username);
        println!("Password: {}", bookmark.password);
    }

    Ok(())
}

fn create_bookmark(args: CreateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut b = Bookmark::new(args.address.clone());
    b.credentials(args.username, args.password);
    b.write_to_file(&args.file)?;

    eprintln!("Wrote bookmark: {}", args.file);

    Ok(())
}
