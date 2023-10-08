use mov_select::{select_movie_show, update_ep_ids};
use crate::hist::{Hist, DataType};
use mov_mod::Mov;
pub mod mov_mod;
mod mov_select;

pub fn search_movie_show(select_provider: bool, vlc: bool) {
    let mut query = String::new();
    println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
    std::io::stdin().read_line(&mut query).expect("reading stdin");
    let mut mov = select_movie_show(&query.replace(" ", "-"));

    main_loop(&mut mov, select_provider, vlc)
}

pub fn select_from_hist(select_provider: bool, vlc: bool) {
    let hist = Hist::deserialize();
    let name = rust_fzf::select(
        hist.mov_data.iter().map(|x| format!("{} Episode {}", x.name, x.ep)).collect(),
        vec![String::from("--reverse")]
    ).split_once(" Episode").unwrap_or_else(|| { println!("{}Exiting...", "\x1b[33m"); std::process::exit(0) })
        .0.to_string();
    let mut mov = hist.mov_data[hist.mov_data.iter().position(|x| x.name == name).unwrap()].clone();
    mov.ep_ids = update_ep_ids(mov.season_id.unwrap());

    main_loop(&mut mov, select_provider, vlc);
}


fn main_loop(mov: &mut Mov, select_provider: bool, vlc: bool) {
    let ep_id = &mov.ep_ids.clone().unwrap()[mov.ep - 1];
    let mut provider_index = get_provider_index(select_provider, &ep_id, mov.clone());

    loop {
        mov.play(provider_index, vlc);
        
        if !mov.name.contains("(movie)") {
            match mov.ep + 1 > mov.ep_ids.clone().unwrap().len() {
                true => {
                    if Hist::deserialize().mov_data.iter().position(|x| x.name == mov.name) != None {
                        Hist::remove(&mov.name, DataType::MovData);
                    }
                },
                false => Hist::mov_save(mov.clone())
            }
        }

        if mov.name.contains("(movie)") {
            let select = rust_fzf::select(
                vec!["search".to_string(), "replay".to_string(), "change provider".to_string(), "quit".to_string()], 
                vec!["--reverse".to_string(), format!("--header={}", mov.name)]).to_string();

            match select.as_str() {
                "search" => {
                    let mut query = String::new();
                    println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
                    std::io::stdin().read_line(&mut query).expect("reading stdin");
                    *mov = select_movie_show(&query.replace(" ", "-"));
                },
                "replay" => (),
                "change provider"  => {
                    provider_index = get_provider_index(true, &ep_id , mov.clone());
                },
                "quit" => std::process::exit(0),
                _ => ()
            }
        } else {
            let select = rust_fzf::select(
                vec![String::from("next"),
                    String::from("replay"),
                    String::from("previous"),
                    String::from("select ep"),
                    String::from("change provider"),
                    String::from("search"),
                    String::from("quit")
                ], 
                vec!["--reverse".to_string(), format!("--header=Current ep - {} of {}", mov.ep, mov.name)]).to_string();

            match select.as_str() {
                "next" => {mov.ep += 1; if mov.ep > mov.ep_ids.clone().unwrap().len() { println!("{}Episode out of bound", "\x1b[31m");  std::process::exit(0) } },
                "replay" => (),
                "previous" => {mov.ep -= 1; if mov.ep == 0 { println!("{}Episode out of bound", "\x1b[31m");  std::process::exit(0) } },
                "select ep" => {
                    mov.ep = rust_fzf::select(
                        (1..=mov.ep_ids.clone().unwrap().len()).map(|x| x.to_string()).collect(),
                        vec!["--reverse".to_string()]
                    ).parse().unwrap()
                },
                "change provider"  => {
                    provider_index = get_provider_index(true, &ep_id , mov.clone());
                },
                "search" => {
                    let mut query = String::new();
                    println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
                    std::io::stdin().read_line(&mut query).expect("reading stdin");
                    *mov = select_movie_show(&query.replace(" ", "-"));
                },
                "quit" => std::process::exit(0),
                _ => ()
            }
        }
    }
}

fn get_provider_index(select_provider: bool, ep_id: &str, mov: Mov) -> usize {
    let range: Vec<String> = match mov.name.contains("(movie)") {
        true => (1..=mov.ep_ids.unwrap().len()).map(|x| x.to_string()).collect(),
        false => (1..=mov_mod::get_ep_data_id(ep_id).len()).map(|x| x.to_string()).collect()
    };

    if range.len() > 1 {
        if select_provider {
            rust_fzf::select(
                range,
                vec![String::from("--reverse"), "--header=Change the provider server. (usualy the last ones don't work) (if you havent changed it it defaults to the first)".to_string()]
            ).parse::<usize>().unwrap_or_else(|_| { println!("{}Exiting...", "\x1b[33m"); std::process::exit(0) }) - 1
        } else { 0 }
    } else { 0 }
}