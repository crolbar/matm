use crate::utils::{get_sources_response, get_response, decrypt_url, Sources};
pub use serde::{Deserialize, Serialize};
use std::process::Command;
use scraper::Selector;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mov {
    pub ep_ids: Option<Vec<u32>>,
    pub season_id: Option<usize>,
    pub name: String,
    pub ep: usize
}

impl Mov {
    pub fn play(&mut self, vlc: bool) {
        match self.get_sources() {
            Ok(sources) => {
                let mut title = format!("{} Episode: {}", self.name, self.ep);
                if self.name.contains("(movie)") { title = format!("{}", self.name.split_once("(movie)").unwrap().0); println!("{}Playing: {}", "\x1b[32m", title) } else { title = title.split("(tv) ").collect::<String>(); println!("{}Playing: {}", "\x1b[32m", title) }
                if vlc {
                    Command::new("vlc")
                            .args([
                                sources.video,
                                format!("--meta-title={}", title)
                            ])
                            .spawn().expect("crashed when trying to start vlc")
                            .wait().unwrap();
                } else {
                    if sources.subs.is_empty() {
                        println!("{}Could't find subtitles", "\x1b[31m");
                        Command::new("mpv")
                            .args([
                                sources.video,
                                format!("--sub-file={}",sources.subs),
                                format!("--force-media-title={}", title),
                                String::from("--fs")
                            ])
                            .spawn().expect("crashed when trying to start mpv")
                            .wait().unwrap();
                    } else {
                        Command::new("mpv")
                            .args([
                                sources.video,
                                format!("--sub-file={}",sources.subs),
                                format!("--force-media-title={}", title),
                                String::from("--fs")
                            ])
                            .spawn().expect("crashed when trying to start mpv")
                            .wait().unwrap();
                    }
                }
            }
            Err(e) => {
                println!("{}Error while trying to get sources: {}", "\x2b[31m", e);
                std::process::exit(1) 
            }
        }
    }

    fn get_sources(&self) -> Result<Sources, Box<dyn std::error::Error>> {
        let mut url = "https://flixhq.to/ajax/sources/".to_string();
        if self.name.contains("movie") {
            url = format!("{}{}", url, self.ep_ids.clone().unwrap()[self.ep - 1])
        } else {
            url = format!("{}{}", url, get_ep_data_id(self.ep_ids.clone().unwrap()[self.ep - 1]))
        }
        
        let provider: Value = serde_json::from_str(get_response(&url)?.as_str())?;
        let provider_url = url::Url::parse(provider["link"].as_str().ok_or("Missing 'link' field")?)?;

        let url = format!("https://{}/ajax/embed-4/getSources?id={}",
            provider_url.host_str().unwrap(),
            provider_url.path().rsplit('/').next().unwrap()
        );

        let sources_json: Value = serde_json::from_str(get_sources_response(&url)?.as_str())?;
        let encrypted = sources_json["encrypted"].as_bool().ok_or("Missing 'encrypted' field")?;
        let mut sub_source = String::new();
        if let Some(english_sub) = sources_json["tracks"].as_array().ok_or("Missing 'tracks' field")?.iter().find(|v| v["label"].to_string().contains("English")) {
            sub_source = english_sub["file"].as_str().unwrap_or_default().to_string();
        }

        if !encrypted {
            let video_url = sources_json.as_str().unwrap().split_once("file\": ").unwrap().1.split_once(",\"type").unwrap().0.to_string();
            Ok(Sources{
                video: video_url,
                subs: sub_source
            })
        } else {
            let enc_video_url = sources_json["sources"].as_str().ok_or("Missing 'sources' field")?.to_string();
            let key: Vec<Vec<u32>> = serde_json::from_str(&get_response(&String::from("https://raw.githubusercontent.com/enimax-anime/key/e4/key.txt"))
                .expect("couldnt get key")).expect("couldnt deserialize string to vec");
            Ok(Sources {
                video: decrypt_url(enc_video_url, key),
                subs: sub_source,
            })
        }
    }
}


fn get_ep_data_id(ep_id: u32) -> u32 {
    let req = scraper::Html::parse_document(&get_response(&format!("https://flixhq.to/ajax/v2/episode/servers/{}", ep_id)).unwrap());
    let a_sel = Selector::parse("a").unwrap();
    req.select(&a_sel).next().unwrap().value().attr("data-id").unwrap().parse::<u32>().unwrap() 
}