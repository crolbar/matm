use crate::utils::{get_sources_response, get_response, decrypt_url, Sources};
pub use serde::{Deserialize, Serialize};
use std::process::Command;
use scraper::Selector;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mov {
    pub ep_ids: Option<Vec<String>>,
    pub season_id: Option<usize>,
    pub name: String,
    pub ep: usize
}


impl Mov {
    pub fn play(&mut self, provider_index: usize ,vlc: bool) {
        match self.get_sources(provider_index) {
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

    fn get_sources(&self, provider_index: usize) -> Result<Sources, Box<dyn std::error::Error>> {
        let mut url = "https://flixhq.to/ajax/sources/".to_string();
        match self.name.contains("movie") {
            true => url = format!("{}{}", url, self.ep_ids.clone().unwrap()[provider_index]),
            false => url = format!("{}{}", url, get_ep_data_id(&self.ep_ids.clone().unwrap()[self.ep - 1])[provider_index])
        }
        
        let provider: Value = serde_json::from_str(get_response(&url)?.as_str())?;
        let provider_url = url::Url::parse(provider["link"].as_str().ok_or("Missing 'link' field")?)?;

        let url = format!("https://{}/ajax/embed-{}/getSources?id={}",
            provider_url.host_str().unwrap(),
            provider_url.path().split_once("embed-").unwrap().1.chars().next().unwrap(),
            provider_url.path().rsplit('/').next().unwrap()
        );
        let response = get_sources_response(&url)?;
        
        
        let sources_json: Value = if serde_json::from_str::<Value>(&response).is_err() {
            println!("{}Couldn't deserialize sources page. Maybe the provier server is down?", "\x1b[31m");
            std::process::exit(1)
        } else { serde_json::from_str(&response).unwrap() };

        let mut sub_source = String::new();
        if let Some(english_sub) = sources_json["tracks"].as_array().ok_or("Missing 'tracks' field")?.iter().find(|v| v["label"].to_string().contains("English")) {
            sub_source = english_sub["file"].as_str().unwrap_or_default().to_string()
        }

        let video_source = if sources_json["encrypted"].as_bool().unwrap() {
            let enc_video_url = sources_json["sources"].as_str().unwrap().to_string();

            let url = format!("https://raw.githubusercontent.com/enimax-anime/key/e{}/key.txt", provider_url.path().split_once("embed-").unwrap().1.chars().next().unwrap());
            let key: Vec<Vec<u32>> = serde_json::from_str(&get_response(&url)
                .expect("couldnt get key")).expect("couldnt deserialize string to vec");

            decrypt_url(enc_video_url, key)
        } else { sources_json["sources"].as_array().unwrap()[0].as_object().unwrap()["file"].as_str().unwrap().to_string() };

        Ok(Sources {
            video: video_source,
            subs: sub_source,
        })
    }
}


pub fn get_ep_data_id(ep_id: &str) -> Vec<String> {
    let req = scraper::Html::parse_document(&get_response(&format!("https://flixhq.to/ajax/v2/episode/servers/{}", ep_id)).unwrap());
    let a_sel = Selector::parse("a").unwrap();
    req.select(&a_sel).map(|x| x.value().attr("data-id").unwrap().to_string()).collect()
}

pub fn update_ep_ids(season_id: usize) -> Option<Vec<String>> {
    let response = get_response(&format!("https://flixhq.to/ajax/v2/season/episodes/{}", season_id)).unwrap();
    let episodes_page = scraper::Html::parse_document(&response);
    let a_sel = Selector::parse("a").unwrap();

    Some(episodes_page.select(&a_sel).map(|x| x.value().attr("data-id").unwrap().to_string()).collect())
}