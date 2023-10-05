use ani_select::{select_anime, update_ep_ids};
use crate::hist::Hist;
use ani_mod::Ani;
mod ani_select;
pub mod ani_mod;

pub fn search_anime() {
    let mut query = String::new();
    println!("{}Search for anime: {}", "\x1b[34m", "\x1b[0m");
    std::io::stdin().read_line(&mut query).expect("reading stdin");
    let mut ani = select_anime(&query);

    main_loop(&mut ani);
}

pub fn select_from_hist() {
    let hist = Hist::deserialize();
    let name = rust_fzf::select(
        hist.ani_data.iter().map(|x| format!("{} Episode {}", x.name, x.ep)).collect(),
        vec![String::from("--reverse")]
    ).split_once(" Episode").unwrap_or_else(|| { println!("{}Exiting...", "\x1b[33m"); std::process::exit(0) })
        .0.to_string();
    let mut ani = hist.ani_data[hist.ani_data.iter().position(|x| x.name == name).unwrap()].clone();
    ani.ep_ids = update_ep_ids(ani.id);

    main_loop(&mut ani);
}


fn main_loop(ani: &mut Ani) {
    loop {
        ani.play();

        match ani.ep + 1 > ani.ep_ids.clone().unwrap().len() {
            true => {
                if Hist::deserialize().ani_data.iter().position(|x| x.name == ani.name) != None {
                    Hist::remove(&ani.name, true);
                }
            },
            false => Hist::ani_save(ani.clone())
        }
        
        let select = rust_fzf::select(
            vec!["next".to_string(), "replay".to_string(), "previous".to_string(), "select ep".to_string(), "search".to_string(), "quit".to_string()], 
            vec!["--reverse".to_string(), format!("--header=Current ep - {} of {}", ani.ep, ani.name)]).to_string();

        match select.as_str() {
            "next" => {ani.ep += 1; if ani.ep > ani.ep_ids.clone().unwrap().len() {println!("{}Episode out of bound", "\x1b[31m"); std::process::exit(0) } },
            "replay" => (),
            "previous" => {ani.ep -= 1; if ani.ep == 0 {println!("{}Episode out of bound", "\x1b[31m"); std::process::exit(0) } },
            "search" => {
                let mut query = String::new();
                println!("{}Search for anime: {}", "\x1b[34m", "\x1b[0m");
                std::io::stdin().read_line(&mut query).expect("reading stdin");
                *ani = select_anime(&query);
            },
            "select ep" => {
                ani.ep = rust_fzf::select(
                    (1..=ani.ep_ids.clone().unwrap().len()).map(|x| x.to_string()).collect(),
                    vec!["--reverse".to_string()]
                ).parse().unwrap()
            },
            "quit" => std::process::exit(0),
            _ => ()
        }
    }
}

pub fn delete_hist() {
    let mut creared = Hist::deserialize();
    creared.ani_data.clear();
    creared.serialize();
    println!("{}History deleted", "\x1b[34m")
}
