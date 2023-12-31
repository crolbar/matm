use clap::Parser;

/// matm 
#[derive(Parser, Debug)]
pub struct Matm {
    #[command(subcommand)]
    pub comm: Option<Comms>
}

#[derive(clap::Subcommand, Debug)]
pub enum Comms {
    /// Watch anime (a for short)
    #[clap(alias = "a")]
    Ani {
        /// Select ep from history
        #[clap(name = "continue", short, long)]
        c: bool,

        /// Delete history
        #[clap(short, long)]
        delete: bool,

        /// Select the provider after you have selected the episode (if not selected it defaults to the first one)
        #[clap(short, long)]
        select_provider: bool,

        /// Watch dubbed
        #[clap(long)]
        dub: bool,
    },
    
    /// Read manga (ma for short)
    #[clap(alias = "ma")]
    Man {
        /// Select ep from history
        #[clap(name = "continue", short, long)]
        c: bool,

        /// Delete history
        #[clap(short, long)]
        delete: bool,

        /// Delete cache
        #[clap(long)]
        clean: bool,
    },

    /// Watch movie/show (m for short)
    #[clap(alias = "m")]
    Mov {
        /// Select ep from history
        #[clap(name = "continue", short, long)]
        c: bool,

        /// Delete history
        #[clap(short, long)]
        delete: bool,

        /// Select the provider after you have selected the episode/movie (if not selected it defaults to the first one)
        #[clap(short, long)]
        select_provider: bool,

        /// Use vlc instead of mpv (not recommended)
        #[clap(short, long)]
        vlc: bool
    },
}

pub struct Sources {
    pub video: String,
    pub subs: String
}

#[tokio::main] 
pub async fn get_response(url: &str) -> Result<String, reqwest::Error> {
    Ok(reqwest::get(url)
       .await?
       .text()
       .await?
    )
}

#[tokio::main] 
pub async fn get_sources_response(url: &str,) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    Ok(client
        .get(url)
        .header("X-Requested-With","XMLHttpRequest")
        .send().await?
        .text().await? 
        .to_string()
    )
}

pub fn decrypt_url(url: String, key: Vec<Vec<u32>>) -> String {
    let mut extracted_key = String::new();
    let mut enc_url: Vec<char> = url.chars().collect();
     
    for i in key {
        for j in i[0]..i[1] {
            extracted_key.push(enc_url[j as usize]);
            enc_url[j as usize] = ' '
        }
    }
    let enc_url_without_key: String = enc_url.iter().filter(|&&c| !c.is_whitespace()).collect();

    let cmd = format!(
        "echo {} | base64 -d | openssl enc -aes-256-cbc -d -md md5 -k {} | sed -nE 's_.*\"file\":\"([^\"]*)\".*_\\1_p'",
        enc_url_without_key, extracted_key
    );

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to decrypt url");

    let decrypted_source = String::from_utf8(output.stdout).expect("Failed to convert to string");

    if !decrypted_source.starts_with("http") {
        println!("{}Could't decrypt video source url", "\x1b[31m");
        std::process::exit(1)
    }

    decrypted_source
}
