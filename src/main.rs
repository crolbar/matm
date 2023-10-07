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
        #[clap(name = "continue", short, long)]
        c: bool,

        /// Delte history
        #[clap(short, long)]
        delete: bool,

        /// Select the provider after you have selected the episode (if not selected it defalts to the first one)
        #[clap(short, long)]
        select_provider: bool,

        /// Watch dubbed
        #[clap(long)]
        dub: bool,
    },
    
    /// Read manga (ma for short)
    #[clap(alias = "ma")]
    Man,

    /// Watch movie/show (m for short)
    #[clap(alias = "m")]
    Mov {
        /// Select ep from history
        #[clap(name = "continue", short, long)]
        c: bool,

        /// Delte history
        #[clap(short, long)]
        delete: bool,

        /// Use vlc instead of mpv (not recommended)
        #[clap(short, long)]
        vlc: bool
    },
}

fn main() {
    let args = Mani::parse();

    match args {
        Mani { comm: Some(comm) } => {
            match comm {

                Comms::Ani { c, delete, select_provider, dub } => {
                    if delete { ani::delete_hist() }
                    else if c { ani::select_from_hist(select_provider, dub) }
                    else { ani::search_anime(select_provider, dub) }
                }

                Comms::Man => man::search_manga(),

                Comms::Mov { c, delete, vlc } => {
                    if delete {  mov::delete_hist() }
                    else if c { mov::select_from_hist(vlc) }
                    else { mov::search_movie_show(vlc) }
                }
            }
        },
    
       _ => {
            match rust_fzf::select(
            vec![String::from("watch anime"), String::from("read manga"), String::from("watch movie/tv show"), String::from("quit")],
            vec![String::from("--reverse")]
            ).as_str() {
                "watch anime" => ani::search_anime(false, false),
                "read manga" => man::search_manga(),
                "watch movie/tv show" => mov::search_movie_show(false),
                "quit" => return,
                _ => ()
            }
       }
    }
}