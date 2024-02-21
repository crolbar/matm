use hist::{Hist, DataType};
use utils::{Matm, Comms};
use clap::Parser;

mod utils;
mod hist;
mod man;
mod mov;
mod ani;

fn main() -> std::io::Result<()> {
    let args = Matm::parse();
    
    match args {
        Matm { comm: Some(comm) } => {
            match comm {
                Comms::Ani { c, delete, select_provider, dub , autoplay} => {
                    if delete { Hist::delete_hist(DataType::AniData) }
                    else if c { ani::select_from_hist(select_provider, dub, autoplay)? }
                    else { ani::search_anime(select_provider, dub, autoplay)? }
                }

                Comms::Man {c, delete, clean } =>{
                    if clean {man::delete_cache()}
                    else if delete { Hist::delete_hist(DataType::ManData) }
                    else if c { man::select_from_hist()?} 
                    else { man::search_manga()? }
                }

                Comms::Mov { c, delete, select_provider, vlc, autoplay } => {
                    if delete {  Hist::delete_hist(DataType::MovData) }
                    else if c { mov::select_from_hist(select_provider, vlc, autoplay)? }
                    else { mov::search_movie_show(select_provider, vlc, autoplay)? }
                }
            }
        },
       
       _ => {
            match selector::select(
                vec![
                    String::from("watch anime"),
                    String::from("read manga"),
                    String::from("watch movie/tv show"),
                    String::from("quit")
                ], None, None,
            )?.as_str() 

            {
                "watch anime" => ani::search_anime(false, false, false)?,
                "read manga" => man::search_manga()?,
                "watch movie/tv show" => mov::search_movie_show(false, false, false)?,
                _ => ()
            }
       }
    }
    Ok(())
}
