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
        #[clap(long)]
        delete: bool,

        /// Select the provider
        #[clap(short, long)]
        select_provider: bool,

        /// Watch dubbed version
        #[clap(short, long)]
        dub: bool,
    
        /// Autoplay next episode (you can stop it by basically hiting ctrl+c in the cli)
        #[clap(short, long)]
        autoplay: bool,
    },
    

    #[cfg(target_os = "linux")]
    /// Read manga (ma for short)
    #[clap(alias = "ma")]
    Man {
        /// Select ep from history
        #[clap(name = "continue", short, long)]
        c: bool,

        /// Delete history
        #[clap(long)]
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
        #[clap(long)]
        delete: bool,

        /// Select the provider
        #[clap(short, long)]
        select_provider: bool,

        /// Use vlc instead of mpv (not recommended)
        #[clap(short, long)]
        vlc: bool,

        /// Autoplay next episode (you can stop it by basically hiting ctrl+c in the cli)
        #[clap(short, long)]
        autoplay: bool,
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

/// 0 is url, 1 is key
pub fn extract_key(url: String, key: Vec<Vec<u32>>) -> (String, String) {
    let mut extracted_key = String::new();
    let mut enc_url: Vec<char> = url.chars().collect();
     
    for i in key {
        for j in i[0]..i[1] {
            extracted_key.push(enc_url[j as usize]);
            enc_url[j as usize] = ' '
        }
    }

    (
        enc_url.iter().filter(|&&c| !c.is_whitespace()).collect(),
        extracted_key
    )
}

use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;

pub fn get_e4_key() -> String {
    let k: Vec<u8> = {
        let resp = get_response("https://keys4.fun").unwrap();

        serde_json::from_str::<Value>(&resp).unwrap()
            ["rabbitstream"].as_object().unwrap()
            ["keys"].as_array().unwrap().iter()
                .map(|i| i.as_u64().unwrap() as u8).collect()
    };
    STANDARD.encode(k)
}

pub fn decrypt_url(enc_sources: String, extracted_key: String) -> String {
    use openssl::{symm::{decrypt, Cipher}, hash::{hash, MessageDigest}};

    let key = {
        let md5 = |input: &[u8]| hash(MessageDigest::md5(), input).unwrap();

        let s = &STANDARD.decode(&enc_sources).unwrap()[8..16];
        let p = extracted_key.as_bytes();
        
        let mut tmp_key = md5(&[&p[..], &s[..]].concat());
        let mut key = tmp_key.clone().to_vec();
        
        while key.len() < 48 {
            tmp_key = md5(&[&tmp_key[..], &p[..], &s[..]].concat());
            key.extend(tmp_key.to_vec())
        }

        key
    };

    let d = decrypt(
        Cipher::aes_256_cbc(),
        &key[..32], Some(&key[32..]),
        &STANDARD.decode(enc_sources).unwrap()[16..].to_vec()
    ).unwrap_or_else(|_| {
        println!("{}Bad decrypt (either aniwatch/flixhd is down or the keys aren't updated)", "\x1b[31m");
        std::process::exit(1)
    });

    serde_json::from_str::<Value>(&String::from_utf8(d).unwrap()).unwrap()
        .as_array().unwrap()[0]
        .as_object().unwrap()
        ["file"].as_str().unwrap()
        .to_string() 
}
