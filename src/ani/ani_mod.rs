use crate::ani::get_response;
use std::process::Command;
use serde_json::Value;

#[derive(Debug)]
pub struct Ani {
    pub ep_ids: Vec<u32>,
    pub name: String,
    pub ep: usize
}

struct Sources {
    video: String,
    subs: String
}

impl Ani {
    pub fn play(&mut self) {
        if let Ok(sources) = self.get_sources() {
            if sources.subs.is_empty() {
                println!("{}Could't find subtitles", "\x1b[31m");
                println!("{}Playing: {} Ep: {}", "\x1b[32m", self.name, self.ep);
                Command::new("mpv")
                    .args([
                        sources.video,
                        format!("--force-media-title={} Episode {}", self.name, self.ep
                    )])
                    .spawn().expect("crashed when trying to start mpv")
                    .wait().unwrap();
            } else {
                println!("{}Playing: {} Ep: {}", "\x1b[32m", self.name, self.ep);
                Command::new("mpv")
                    .args([
                        sources.video,
                        format!("--sub-file={}",sources.subs),
                        format!("--force-media-title={} Episode {}", self.name, self.ep
                    )])
                    .spawn().expect("crashed when trying to start mpv")
                    .wait().unwrap();
            }
        } else { println!("{}Error while trying to get sources", "\x1b[31m") } 
    }


    fn get_sources(&self) -> Result<Sources, Box<dyn std::error::Error>> {
        let url = format!("https://aniwatch.to/ajax/v2/episode/sources?id={}", get_ep_data_id(self.ep_ids[self.ep - 1]));
        let provider: Value = serde_json::from_str(get_response(url)?.as_str())?;
        let provider_url = url::Url::parse(provider["link"].as_str().ok_or("Missing 'link' field")?)?;

        let url = format!("https://{}/embed-2/ajax/e-1/getSources?id={}",
            provider_url.host_str().unwrap(),
            provider_url.path().rsplit('/').next().unwrap()
        );

        let sources_json: Value = serde_json::from_str(get_response(url)?.as_str())?;
        let enc_video_url = sources_json["sources"].as_str().ok_or("Missing 'sources' field")?.to_string();
        let encrypted = sources_json["encrypted"].as_bool().ok_or("Missing 'encrypted' field")?;

        let mut sub_source = String::new();
        if let Some(english_sub) = sources_json["tracks"].as_array().ok_or("Missing 'tracks' field")?.iter().find(|v| v["label"] == "English") {
            sub_source = english_sub["file"].as_str().unwrap_or_default().to_string();
        }

        if !encrypted {
            let video_url = enc_video_url.split_once("file:").unwrap().1.split_once(",type").unwrap().0.to_string();
            Ok(Sources{
                video: video_url,
                subs: sub_source
            })
        } else {
            Ok(Sources {
                video: decrypt_url(enc_video_url),
                subs: sub_source,
            })
        }
    }
}

fn decrypt_url(url: String) -> String {
    let key: Vec<Vec<u32>> = serde_json::from_str(
        &get_response(String::from("https://raw.githubusercontent.com/enimax-anime/key/e6/key.txt"))
        .expect("couldnt get key")).expect("couldnt deserialize string to vec");
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
        "echo {} | base64 -d | openssl enc -aes-256-cbc -d -md md5 -k {} 2>/dev/null | sed -nE 's_.*\"file\":\"([^\"]*)\".*_\\1_p'",
        enc_url_without_key, extracted_key
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute command");

    let decrypted_source = String::from_utf8(output.stdout).expect("Failed to convert to string");


    if !decrypted_source.starts_with("http") {
        println!("{}Could't decrypt video source url", "\x1b[31m")
    }


    decrypted_source
}


fn get_ep_data_id(ep_id: u32) -> u32 {
    let url = format!("https://aniwatch.to/ajax/v2/episode/servers?episodeId={}", ep_id);
    let data_ids_req = get_response(url).unwrap();
    data_ids_req.split_once("data-id=\\\"").unwrap().1
        .split_once("\\\"\\n").unwrap().0
        .parse::<u32>().unwrap()
}