use mov_select::{select_movie_show, update_ep_ids};
use crate::hist::Hist;
use mov_mod::Mov;
pub mod mov_mod;
mod mov_select;

pub fn search_movie_show() {
    let mut query = String::new();
    println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
    std::io::stdin().read_line(&mut query).expect("reading stdin");
    let mut mov = select_movie_show(&query.replace(" ", "-"));

    main_loop(&mut mov)
}

pub fn select_from_hist() {
    let hist = Hist::deserialize();
    let name = rust_fzf::select(
        hist.mov_data.iter().map(|x| format!("{} Episode {}", x.name, x.ep)).collect(),
        vec![String::from("--reverse")]
    ).split_once(" Episode").unwrap_or_else(|| { println!("{}Exiting...", "\x1b[33m"); std::process::exit(0) })
        .0.to_string();
    let mut mov = hist.mov_data[hist.mov_data.iter().position(|x| x.name == name).unwrap()].clone();
    mov.ep_ids = update_ep_ids(mov.season_id.unwrap());

    main_loop(&mut mov);
}


fn main_loop(mov: &mut Mov) {
    loop {
        mov.play();

        match mov.ep + 1 > mov.ep_ids.clone().unwrap().len() {
            true => {
                if Hist::deserialize().mov_data.iter().position(|x| x.name == mov.name) != None {
                    Hist::remove(&mov.name, false);
                }
            },
            false => Hist::mov_save(mov.clone())
        }

        if mov.name.contains("(movie)") {
            let select = rust_fzf::select(
                vec!["search".to_string(), "quit".to_string()], 
                vec!["--reverse".to_string(), format!("--header={}", mov.name)]).to_string();

            match select.as_str() {
                "search" => {
                    let mut query = String::new();
                    println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
                    std::io::stdin().read_line(&mut query).expect("reading stdin");
                    *mov = select_movie_show(&query.replace(" ", "-"));
                },
                "quit" => std::process::exit(0),
                _ => ()
            }
        } else {
            let select = rust_fzf::select(
                vec!["next".to_string(), "replay".to_string(), "previous".to_string(),"select ep".to_string(), "search".to_string(), "quit".to_string()], 
                vec!["--reverse".to_string(), format!("--header=Current ep - {} of {}", mov.ep, mov.name)]).to_string();

            match select.as_str() {
                "next" => {mov.ep += 1; if mov.ep > mov.ep_ids.clone().unwrap().len() { println!("{}Episode out of bound", "\x1b[31m");  std::process::exit(0) } },
                "replay" => (),
                "previous" => {mov.ep -= 1; if mov.ep == 0 { println!("{}Episode out of bound", "\x1b[31m");  std::process::exit(0) } },
                "search" => {
                    let mut query = String::new();
                    println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
                    std::io::stdin().read_line(&mut query).expect("reading stdin");
                    *mov = select_movie_show(&query.replace(" ", "-"));
                },
                "select ep" => {
                    mov.ep = rust_fzf::select(
                        (1..=mov.ep_ids.clone().unwrap().len()).map(|x| x.to_string()).collect(),
                        vec!["--reverse".to_string()]
                    ).parse().unwrap()
                },
                "quit" => std::process::exit(0),
                _ => ()
            }
        }
    }
}

pub fn delete_hist() {
    let mut creared = Hist::deserialize();
    creared.mov_data.clear();
    creared.serialize();
    println!("{}History deleted", "\x1b[34m")
}