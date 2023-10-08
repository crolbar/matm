use crate::hist::{Hist, DataType};
use man_select::select_manga;
use man_mod::Man;
pub mod man_mod;
mod man_select;

pub fn search_manga() {
    let mut query = String::new();
    println!("{}Search for manga: {}", "\x1b[34m", "\x1b[0m");
    std::io::stdin().read_line(&mut query).expect("reading stdin");
    let mut man: Man = select_manga(&query.replace(" ", "_"));
    
    main_loop(&mut man)
}

pub fn select_from_hist() {
    let hist = Hist::deserialize();
    let name = rust_fzf::select(
        hist.man_data.iter().map(|x| format!("{} Episode {}", x.name, x.chapter)).collect(),
        vec![String::from("--reverse")]
    ).split_once(" Episode").unwrap_or_else(|| { println!("{}Exiting...", "\x1b[33m"); std::process::exit(0) })
        .0.to_string();
    let mut man = hist.man_data[hist.man_data.iter().position(|x| x.name == name).unwrap()].clone();
    man.all_chapters = Man::get_all_chapters(&man.url_id);

    main_loop(&mut man);
}


fn main_loop(man: &mut Man) {
    if std::fs::metadata(dirs::home_dir().unwrap().join(".cache/mani")).is_err() { std::fs::create_dir_all(dirs::home_dir().unwrap().join(".cache/mani")).unwrap() }
    if std::fs::metadata(dirs::home_dir().unwrap().join(".cache/mani/false.cbz")).is_ok() { std::fs::remove_file(dirs::home_dir().unwrap().join(".cache/mani/false.cbz")).unwrap() }
    loop {
        man.read();
        let current_chapter_index = man.all_chapters.iter().position(|x| x == &man.chapter).unwrap();


        match &man.chapter >= man.all_chapters.last().unwrap() {
            true => {
                if Hist::deserialize().man_data.iter().position(|x| x.name == man.name) != None {
                    Hist::remove(&man.name, DataType::ManData);
                }
            },
            false => Hist::man_save(man.clone())
        }

        if &man.chapter < man.all_chapters.last().unwrap() {
            let mut tmp_man = man.clone();
            tmp_man.chapter = tmp_man.all_chapters[current_chapter_index + 1].clone();
            tmp_man.create_cbz();
        }

        let select = rust_fzf::select(
            vec![String::from("next"),String::from("reload"), String::from("previous"), String::from("select chapter"), String::from("search"), String::from("quit")],
            vec![String::from("--reverse"), format!("--header=Current chapter - {} of {}", man.chapter, man.name)]
        );

        match select.as_str() {
            "next" => {
                if &man.chapter >= man.all_chapters.last().unwrap() { 
                    println!("{}Episode out of bound", "\x1b[31m"); 
                    std::process::exit(0) 
                };
                man.chapter = man.all_chapters[current_chapter_index + 1].clone()
            },
            "reload" => std::fs::remove_file(dirs::home_dir().unwrap().join(format!(".cache/mani/{}-{}.cbz", man.name, man.chapter))).unwrap(),
            "previous" => {
                if man.chapter <= 0.0 { 
                    println!("{}Episode out of bound", "\x1b[31m");
                    std::process::exit(0) 
                }; 
                man.chapter = man.all_chapters[current_chapter_index - 1].clone()
            },
            "select chapter" => man.chapter = rust_fzf::select(man.all_chapters.clone().iter().map(|x| x.to_string()).collect(), vec!["--reverse".to_string()]).parse().unwrap(),
            "search" => { 
                let mut query = String::new();
                println!("{}Search for manga: {}", "\x1b[34m", "\x1b[0m");
                std::io::stdin().read_line(&mut query).expect("reading stdin");
                *man = select_manga(&query.replace(" ", "_"));
            },
            _ => std::process::exit(0)
        }
    }
}

pub fn delete_cache() {
    std::fs::remove_dir_all(dirs::home_dir().unwrap().join(".cache/mani")).unwrap();
    std::fs::create_dir(dirs::home_dir().unwrap().join(".cache/mani")).unwrap();
    println!("{}Cache cleared", "\x1b[34m")
}