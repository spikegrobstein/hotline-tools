use hotline_bookmark::bookmark::Bookmark;

fn main() {
    let mut b2 = Bookmark::new("24.30.100.120".into());
    b2.credentials("star".into(), "guest".into());
    b2.write_to_file("./new_bookmark.hlbm").unwrap();

    let b = Bookmark::from_file("./new_bookmark.hlbm").unwrap();

    println!("{:?}", b);

}
