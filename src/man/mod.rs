use man::select_manga;
mod man;

pub fn search_manga() {
    if std::fs::metadata("/tmp/mani").is_err() { std::fs::create_dir_all("/tmp/mani/imgs").unwrap() }
    
    let mut query = String::new();
    println!("{}Search for manga: {}", "\x1b[34m", "\x1b[0m");
    std::io::stdin().read_line(&mut query).expect("reading stdin");

    let mut man = select_manga(&query.replace(" ", "_"));

    man.select_chapter();
    
    man.read();
}

