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
    pub fn play(&mut self) {
        let mut data_id = get_ep_data_id(self.ep_ids.clone().unwrap()[self.ep - 1]);

        match get_sources(&mut data_id) {
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

fn get_sources(data_id: &mut  Vec<u32>) -> Result<Sources, Box<dyn std::error::Error>> {
    let url = format!("https://aniwatch.to/ajax/v2/episode/sources?id={}", data_id[0]);
    let provider: Value = serde_json::from_str(get_response(url)?.as_str())?;
    let provider_url = url::Url::parse(provider["link"].as_str().ok_or("Missing 'link' field")?)?;

    let url = format!("https://{}/embed-2/ajax/e-1/getSources?id={}",
        provider_url.host_str().unwrap(),
        provider_url.path().rsplit('/').next().unwrap()
    );

    let sources_json: Value = serde_json::from_str(get_response(url)?.as_str()).expect("Couldn't deserialize sources page. Maybe the provier server is down?");
    let enc_video_url = sources_json["sources"].as_str().ok_or("Missing 'sources' field")?.to_string();
    let encrypted = sources_json["encrypted"].as_bool().ok_or("Missing 'encrypted' field")?;

    let mut sub_source = String::new();
    if let Some(english_sub) = sources_json["tracks"].as_array().ok_or("Missing 'tracks' field")?.iter().find(|v| v["label"] == "English") {
        sub_source = english_sub["file"].as_str().unwrap_or_default().to_string();
    }
    
    if enc_video_url.is_empty() {
        data_id.remove(0);
        if data_id.is_empty() {
            println!("{}Couldn't get the video sources. Maybe all the provider servers are down?", "\x1b[31m");
            std::process::exit(1)
        }

        return get_sources(data_id);
    }

    if !encrypted {
        let video_url = enc_video_url.split_once("file:").unwrap().1.split_once(",type").unwrap().0.to_string();
        Ok(Sources{
            video: video_url,
            subs: sub_source
        })
    } else {
        let key: Vec<Vec<u32>> = serde_json::from_str(&get_response(String::from("https://raw.githubusercontent.com/enimax-anime/key/e6/key.txt"))
            .expect("couldnt get key")).expect("couldnt deserialize string to vec");
        Ok(Sources {
            video: decrypt_url(enc_video_url, key),
            subs: sub_source,
        })
    }
}

fn get_ep_data_id(ep_id: u32) -> Vec<u32> {
    let response = get_response(format!("https://aniwatch.to/ajax/v2/episode/servers?episodeId={}", ep_id)).unwrap();
    response.split("data-type=\\\"sub\\\" data-id=\\\"").skip(1)
        .map(|x| x.split_once("\\\"\\n").unwrap().0.parse().unwrap())
        .collect()
}