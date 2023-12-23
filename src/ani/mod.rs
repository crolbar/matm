use ani_select::{select_anime, update_ep_ids};
use crate::hist::{Hist, DataType};
pub use ani_mod::Ani;
mod ani_select;
mod ani_mod;

pub fn search_anime(select_provider: bool, is_dub: bool) -> std::io::Result<()> {
    let mut query = String::new();

    while query.trim().is_empty() {
        println!("{}Search for anime: {}", "\x1b[34m", "\x1b[0m");
        std::io::stdin().read_line(&mut query).expect("reading stdin");
    }

    let mut ani = select_anime(&query)?;

    Ok(main_loop(&mut ani, select_provider, is_dub)?)
}

pub fn select_from_hist(select_provider: bool, is_dub: bool) -> std::io::Result<()> {
    let hist = Hist::deserialize();

    let name = selector::select(
            hist.ani_data
            .iter()
            .map(|x| format!("{} Episode {}", x.name, x.ep))
            .collect(),
            None, None
        )?.split_once(" Episode")
            .unwrap_or_else(|| {
                println!("{}Exiting...", "\x1b[33m");
                std::process::exit(0) 
            }).0.to_string();
    
    let mut ani = hist.ani_data.iter().find(|x| x.name == name).unwrap().clone();

    ani.ep_ids = update_ep_ids(ani.id);

    Ok(main_loop(&mut ani, select_provider, is_dub)?)
}

fn main_loop(ani: &mut Ani, select_provider: bool, is_dub: bool) -> std::io::Result<()> {
    let mut is_dub = is_dub;
    let ep_id = ani.ep_ids.clone().unwrap()[ani.ep.clone() - 1];
    let mut provider_index = get_provider_index(select_provider, &ep_id, is_dub)?;

    loop {
        ani.play(provider_index, is_dub);

        match ani.ep + 1 > ani.ep_ids.clone().unwrap().len() {
            true => {
                if Hist::deserialize().ani_data.iter().position(|x| x.name == ani.name) != None {
                    Hist::remove(&ani.name, DataType::AniData);
                }
            },
            false => Hist::ani_save(ani.clone())
        }
        
        let select = selector::select(
            vec![String::from("next"),
                String::from("replay"),
                String::from("previous"),
                String::from("select ep"),
                format!("switch to {}", if is_dub {"sub"} else {"dub"}),
                String::from("change provider"),
                String::from("search"),
                String::from("quit")
            ], 
            Some(&format!("Current ep - {} of {}", ani.ep, ani.name)), None
        )?;

        match select.as_str() {
            "next" => {ani.ep += 1; if ani.ep > ani.ep_ids.clone().unwrap().len() {println!("{}Episode out of bound", "\x1b[31m"); std::process::exit(0) } },
            "replay" => (),
            "previous" => {ani.ep -= 1; if ani.ep == 0 {println!("{}Episode out of bound", "\x1b[31m"); std::process::exit(0) } },
            "select ep" => {
                ani.ep = selector::select(
                    (1..=ani.ep_ids.clone().unwrap().len()).map(|x| x.to_string()).collect(),
                    None, None
                )?.parse().unwrap_or_else(|_| { println!("{}Exiting...", "\x1b[33m"); std::process::exit(0) })
            },
            "switch to sub" => is_dub = false,
            "switch to dub" => is_dub = true,
            "change provider"  => {
                provider_index = get_provider_index(true, &ep_id, is_dub)?;
            },
            "search" => {
                let mut query = String::new();
                println!("{}Search for anime: {}", "\x1b[34m", "\x1b[0m");
                std::io::stdin().read_line(&mut query).expect("reading stdin");
                *ani = select_anime(&query)?
            },
            "quit" => std::process::exit(0),
            _ => ()
        }
    }
}

fn get_provider_index(select_provider: bool, ep_id: &u32, is_dub: bool) -> std::io::Result<usize> {
    if select_provider {
        Ok(
            selector::select(
                (1..=ani_mod::get_ep_data_id(ep_id, is_dub).len()).map(|x| x.to_string()).collect(),
                Some(
                    "Change the provider server. (usualy the last ones are not supported) (if you havent changed it, it defaults to the first)"
                ), None
            )?
            .parse::<usize>().unwrap_or_else(|_| {
                println!("{}Exiting...", "\x1b[33m");
                std::process::exit(0) 
            }) - 1
        )
    } else { Ok(0) }
}
