use crate::hist::{Hist, DataType};
use man_select::select_manga;
pub use man_mod::Man;
mod man_mod;
mod man_select;

pub fn search_manga() -> std::io::Result<()> {
    let mut query = String::new();

    while query.trim().is_empty() {
        println!("{}Search for manga: {}", "\x1b[34m", "\x1b[0m");
        std::io::stdin().read_line(&mut query).expect("reading stdin");
    }

    let mut man: Man = select_manga(&query.replace(" ", "_"))?;
    
    Ok(main_loop(&mut man)?)
}

pub fn select_from_hist() -> std::io::Result<()> {
    let hist = Hist::deserialize();

    let name = selector::select(
        hist.man_data
        .iter()
        .map(|x| format!("{} Chapter: {}", x.name, x.chapter))
        .collect(),
        None, None
    )?.split_once(" Chapter")
        .unwrap_or_else(|| {
            println!("{}Exiting...", "\x1b[33m");
            std::process::exit(0) 
        }).0.to_string();

    let mut man: Man = hist.man_data.iter().find(|m| m.name == name).unwrap().clone();

    man.all_chapters = Man::get_all_chapters(&man.url_id);

    Ok(main_loop(&mut man)?)
}


fn main_loop(man: &mut Man) -> std::io::Result<()> {
    check_missing_dirs();

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

        let select = selector::select(
            vec![String::from("next"),
                String::from("reload"),
                String::from("previous"),
                String::from("select chapter"),
                String::from("search"),
                String::from("quit")
            ],
            Some(format!("--header=Current chapter - {} of {}", man.chapter, man.name).as_str()), None
        )?;

        match select.as_str() {
            "next" => {
                if &man.chapter >= man.all_chapters.last().unwrap() { 
                    println!("{}Episode out of bound", "\x1b[31m"); 
                    std::process::exit(0) 
                };

                man.chapter = man.all_chapters[current_chapter_index + 1].clone();
            },
            "reload" => std::fs::remove_file(dirs::home_dir().unwrap().join(format!(".cache/matm/{}-{}.cbz", man.name, man.chapter))).unwrap(),
            "previous" => {
                if man.chapter <= 0.0 { 
                    println!("{}Episode out of bound", "\x1b[31m");
                    std::process::exit(0) 
                }; 

                man.chapter = man.all_chapters[current_chapter_index - 1].clone()
            },
            "select chapter" => {
                man.chapter = selector::select(
                    man.all_chapters.iter().map(|x| x.to_string()).collect(),
                    None, None
                )?.parse().unwrap();
            },
            "search" => { 
                let mut query = String::new();
                println!("{}Search for manga: {}", "\x1b[34m", "\x1b[0m");
                std::io::stdin().read_line(&mut query).expect("reading stdin");
                *man = select_manga(&query.replace(" ", "_"))?;
            },
            _ => std::process::exit(0)
        }
    }
}

pub fn delete_cache() {
    std::fs::remove_dir_all(dirs::home_dir().unwrap().join(".cache/matm")).unwrap();
    std::fs::create_dir(dirs::home_dir().unwrap().join(".cache/matm")).unwrap();
    println!("{}Cache cleared", "\x1b[34m")
}

fn check_missing_dirs() {
    let home_dir = dirs::home_dir().unwrap();

    if std::fs::metadata(home_dir.join(".cache/matm")).is_err() { 
        std::fs::create_dir_all(home_dir.join(".cache/matm")).unwrap() 
    }

    if std::fs::metadata(home_dir.join(".cache/matm/false.cbz")).is_ok() {
        std::fs::remove_file(home_dir.join(".cache/matm/false.cbz")).unwrap() 
    }
}
