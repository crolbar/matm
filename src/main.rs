use clap::Parser;
mod utils;
mod hist;
mod man;
mod mov;
mod ani;

/// mani
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Mani {
    #[command(subcommand)]
    comm: Option<Comms>
}

#[derive(clap::Subcommand, Debug)]
enum Comms {
    /// Watch anime (a for short)
    #[clap(alias = "a")]
    Ani {
        /// Select ep from history
        #[clap(short)]
        c: bool,

        /// Delte history
        #[clap(short)]
        d: bool
    },
    
    /// Read manga (ma for short)
    #[clap(alias = "ma")]
    Man,

    /// Watch movie/show (m for short)
    #[clap(alias = "m")]
    Mov {
        /// Select ep from history
        #[clap(short)]
        c: bool,

        /// Delte history
        #[clap(short)]
        d: bool
    },
}

fn main() {
    let args = Mani::parse();

    match args {
       Mani { comm: Some(Comms::Ani { c: false, d: true }) } => ani::delete_hist(),
       Mani { comm: Some(Comms::Ani { c: true, d: false }) } => ani::select_from_hist(),
       Mani { comm: Some(Comms::Ani { c: false, d: false}) } => ani::search_anime(),
       Mani { comm: Some(Comms::Man) } => man::search_manga(),
       Mani { comm: Some(Comms::Mov { c: false, d: true }) } => mov::delete_hist(),
       Mani { comm: Some(Comms::Mov { c: true, d: false }) } => mov::select_from_hist(),
       Mani { comm: Some(Comms::Mov { c: false, d: false}) } => mov::search_movie_show(),
       _ => {
            match rust_fzf::select(
            vec![String::from("watch anime"), String::from("read manga"), String::from("watch movie/tv show"), String::from("quit")],
            vec![String::from("--reverse")]
            ).as_str() {
                "watch anime" => ani::search_anime(),
                "read manga" => man::search_manga(),
                "watch movie/tv show" => mov::search_movie_show(),
                "quit" => return,
                _ => ()
            }
       }
    }
}
