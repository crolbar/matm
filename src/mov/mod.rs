use crate::hist::{Hist, DataType};
pub use mov_mod::Mov;
mod mov_mod;
mod mov_select;

pub fn search_movie_show(select_provider: bool, vlc: bool) -> std::io::Result<()> {
    let mut query = String::new();

    while query.trim().is_empty() {
        println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
        std::io::stdin().read_line(&mut query).expect("reading stdin");
    }

    let mut mov = Mov::select_movie_show(&query.replace(" ", "-"))?;

    Ok(main_loop(&mut mov, select_provider, vlc)?)
}

pub fn select_from_hist(select_provider: bool, vlc: bool) -> std::io::Result<()> {
    let hist = Hist::deserialize();

    let name = 
        selector::select(
            hist.mov_data
                .iter()
                .map(|x| {
                    format!("{} Episode {}", x.name, x.ep)
                })
                .collect(),
                None, None
        )?.split_once(" Episode")
            .unwrap_or_else(|| {
                println!("{}Exiting...", "\x1b[33m");
                std::process::exit(0) 
            }).0
        .to_string();

    let mut mov = hist.mov_data.iter().find(|m| m.name == name).unwrap().clone();

    mov.update_ep_ids();

    Ok(main_loop(&mut mov, select_provider, vlc)?)
}


fn main_loop(mov: &mut Mov, select_provider: bool, vlc: bool) -> std::io::Result<()> {
    let mut provider_index = get_provider_index(select_provider, mov)?;

    mov.play(provider_index, vlc);

    loop {
        save_to_hist(mov);

        if mov.name.contains("(movie)") {
            let select = selector::select(
                vec![String::from("search"),
                    String::from("replay"),
                    String::from("change provider"),
                    String::from("quit"),
                ], 
               Some(&mov.name), None
           )?;

            match select.as_str() {
                "search" => {
                    let mut query = String::new();
                    println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
                    std::io::stdin().read_line(&mut query).expect("reading stdin");
                    *mov = Mov::select_movie_show(&query.replace(" ", "-"))?;
                },
                "change provider"  => {
                    provider_index = get_provider_index(true, mov)?;
                },
                "quit" => std::process::exit(0),
                _ => ()
            }
        } else {
            let select = selector::select(
                vec![String::from("play next"),
                    String::from("play"),
                    String::from("next"),
                    String::from("previous"),
                    String::from("select ep"),
                    String::from("change provider"),
                    String::from("search"),
                    String::from("quit")
                ], 
                Some(&format!("Current ep - {} of {}", mov.ep, mov.name)), None
            )?;

            match select.as_str() {
                "play next" => {
                    mov.ep += 1;

                    if mov.ep > mov.ep_ids.clone().unwrap().len() {
                        println!("{}Episode out of bound", "\x1b[31m");
                        std::process::exit(0) 
                    } 

                    mov.play(provider_index, vlc);
                }
                "play" => {
                    mov.play(provider_index, vlc);
                },
                "next" => {
                    mov.ep += 1;

                    if mov.ep > mov.ep_ids.clone().unwrap().len() {
                        println!("{}Episode out of bound", "\x1b[31m");
                        std::process::exit(0) 
                    } 
                },
                "previous" => {
                    mov.ep -= 1; 

                    if mov.ep == 0 {
                        println!("{}Episode out of bound", "\x1b[31m");
                        std::process::exit(0) 
                    } 
                },
                "select ep" => {
                    mov.ep = selector::select(
                        (1..=mov.ep_ids.clone().unwrap().len()).map(|x| x.to_string()).collect(),
                        Some("select episode"), None
                    )?.parse().unwrap()
                },
                "change provider"  => {
                    provider_index = get_provider_index(true, mov)?;
                },
                "search" => {
                    let mut query = String::new();
                    println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
                    std::io::stdin().read_line(&mut query).expect("reading stdin");
                    *mov = Mov::select_movie_show(&query.replace(" ", "-")).unwrap();
                },
                "quit" => {
                    std::process::exit(0);
                } 
                _ => ()
            }
        }
    }
}

fn get_provider_index(select_provider: bool, mov: &Mov) -> std::io::Result<usize> {
    let range: Vec<String> = match mov.name.contains("(movie)") {
        true => 1..=mov.ep_ids.clone().unwrap().len(),
        false => 1..=mov.get_ep_data_id().len()
    }.map(|x| x.to_string()).collect();

    if range.len() > 1 {
        if select_provider {
            Ok(
                selector::select(
                    range,
                    Some("Change the provider server. (usualy the last ones are not supported) (if you havent changed it, it defaults to the first)"), None
                ).unwrap()
                .parse::<usize>().unwrap_or_else(|_| {
                    println!("{}Exiting...", "\x1b[33m");
                    std::process::exit(0) 
                }) - 1
            )
        } else { Ok(0) }
    } else { Ok(0) }
}

fn save_to_hist(mov: &Mov) {
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
}
