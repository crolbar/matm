use clap::Parser;
mod man;
mod ani;
mod mov;

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
    Ani,
    
    /// Read manga (ma for short)
    #[clap(alias = "ma")]
    Man,

    /// Watch movie/show (m for short)
    #[clap(alias = "m")]
    Mov,
}

fn main() {
    let args = Mani::parse();

    match args {
       Mani { comm: Some(Comms::Ani) } => ani::search_anime(),
       Mani { comm: Some(Comms::Man) } => man::search_manga(),
       Mani { comm: Some(Comms::Mov) } => mov::search_movie(),
       _ => {
            match rust_fzf::select(
            vec![String::from("watch anime"), String::from("read manga"), String::from("watch movie/tv show"), String::from("quit")],
            vec![String::from("--reverse")]
            ).as_str() {
                "watch anime" => ani::search_anime(),
                "read manga" => man::search_manga(),
                "watch movie/tv show" => mov::search_movie(),
                "quit" => return,
                _ => ()
            }
       }
    }
}
