use hotline_bookmark::bookmark::Bookmark;

// usage:
// hlbm print <bookmark>
// hlbm create <bookmark> <address> --username <username> --password <password>
// --json for json printing
// use `-` for <address> if using json on STDIN (from output from tracker)
// ~/.config/hlbm/hooks.json -- for hooks for connecting?

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);

    for b in args {
        println!("Opening {b}");
        let bookmark = Bookmark::from_file(&b).unwrap();
        println!("{:#?}", bookmark);

    }
    // let mut b2 = Bookmark::new("24.30.100.120".into());
    // b2.credentials("star".into(), "guest".into());
    // b2.write_to_file("./new_bookmark.hlbm").unwrap();

    // let b = Bookmark::from_file("./new_bookmark.hlbm").unwrap();

    // println!("{:?}", b);

}
