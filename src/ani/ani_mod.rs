use crate::utils::{get_response, decrypt_url, Sources};
use serde::{Deserialize, Serialize};
use std::process::Command;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ani {
    pub ep_ids: Option<Vec<u32>>,
    pub name: String,
    pub ep: usize,
    pub id: usize
}


impl Ani {
    pub fn play(&mut self, provider_index: usize, is_dub: bool) {
        let data_id = get_ep_data_id(&self.ep_ids.clone().unwrap()[self.ep - 1], is_dub);
        let data_id = if provider_index >= data_id.len() {
            println!("{}This episode dosnt have an dubbed version", "\x1b[31m");
            get_ep_data_id(&self.ep_ids.clone().unwrap()[self.ep - 1], !is_dub)
        } else { data_id };

        match get_sources(&data_id[provider_index]) {
            Ok(sources) => {
                if sources.subs.is_empty() {
                    println!("{}Could't find subtitles", "\x1b[31m");
                    println!("{}Playing: {} Ep: {}", "\x1b[32m", self.name, self.ep);
                    Command::new("mpv")
                        .args([
                            sources.video,
                            format!("--force-media-title={} Episode {}", self.name, self.ep),
                            String::from("--fs")
                        ])
                        .spawn().expect("crashed when trying to start mpv")
                        .wait().unwrap();
                } else {
                    println!("{}Playing: {} Ep: {}", "\x1b[32m", self.name, self.ep);
                    Command::new("mpv")
                        .args([
                            sources.video,
                            format!("--sub-file={}",sources.subs),
                            format!("--force-media-title={} Episode {}", self.name, self.ep),
                            String::from("--fs")
                        ])
                        .spawn().expect("crashed when trying to start mpv")
                        .wait().unwrap();
                }
            }
            Err(e) => {
                println!("{}Error while trying to get sources: {}", "\x1b[31m", e);
                std::process::exit(1) 
            }
        }
    }
}

fn get_sources(data_id: &str) -> Result<Sources, Box<dyn std::error::Error>> {
    let url = format!("https://aniwatch.to/ajax/v2/episode/sources?id={}", data_id);
    let provider: Value = serde_json::from_str(get_response(&url)?.as_str())?;
    let provider_url = url::Url::parse(provider["link"].as_str().ok_or("Missing 'link' field")?)?;

    let url = format!("https://{}/embed-2/ajax/e-1/getSources?id={}",
        provider_url.host_str().unwrap(),
        provider_url.path().rsplit('/').next().unwrap()
    );
    let response = get_response(&url)?;

    let sources_json: Value = if serde_json::from_str::<Value>(response.as_str()).is_err() {
        println!("{}Couldn't deserialize sources page. Maybe the provier server is down?", "\x1b[31m");
        std::process::exit(1)
    } else { serde_json::from_str(response.as_str()).unwrap() };



    let mut sub_source = String::new();
    if let Some(english_sub) = sources_json["tracks"].as_array().ok_or("Missing 'tracks' field")?.iter().find(|v| v["label"] == "English") {
        sub_source = english_sub["file"].as_str().unwrap_or_default().to_string();
    };

    let video_source = if sources_json["encrypted"].as_bool().unwrap() {
        let enc_video_url = sources_json["sources"].as_str().unwrap().to_string();

        let url = format!("https://raw.githubusercontent.com/theonlymo/keys/e{}/key", provider_url.path().split_once("e-").unwrap().1.chars().next().unwrap());
        let key: Vec<Vec<u32>> = serde_json::from_str(&get_response(&url)
            .expect("couldnt get key")).expect("couldnt deserialize string to vec");

        decrypt_url(enc_video_url, key)
    } else { sources_json["sources"].as_array().unwrap()[0].as_object().unwrap()["file"].as_str().unwrap().to_string() };

    Ok(Sources {
        video: video_source,
        subs: sub_source,
    })
}


pub fn get_ep_data_id(ep_id: &u32, is_dub: bool) -> Vec<String> {
    let video_type = if is_dub {"dub"} else {"sub"};
    let response = get_response(&format!("https://aniwatch.to/ajax/v2/episode/servers?episodeId={}", ep_id)).unwrap();
    response.split(format!("data-type=\\\"{}\\\" data-id=\\\"", video_type).as_str()).skip(1)
        .map(|x| x.split_once("\\\"\\n").unwrap().0.to_string())
        .collect()
}
