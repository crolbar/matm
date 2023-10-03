use crate::mov::get_response;
use std::process::Command;
use scraper::Selector;
use serde_json::Value;

#[derive(Debug)]
pub struct Mov {
    pub ep_ids: Vec<u32>,
    pub name: String,
    pub ep: usize
}

struct Sources {
    video: String,
    subs: String
}

impl Mov {
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
        } else { println!("{}Error while trying to get sources", "\x1b[31m"); std::process::exit(1) } 
    }


    fn get_sources(&self) -> Result<Sources, Box<dyn std::error::Error>> {
        let url = format!("https://flixhq.to/ajax/sources/{}", get_ep_data_id(self.ep_ids[self.ep - 1]));
        let provider: Value = serde_json::from_str(get_response(url)?.as_str())?;
        let provider_url = url::Url::parse(provider["link"].as_str().ok_or("Missing 'link' field")?)?;

        let url = format!("https://{}/ajax/embed-4/getSources?id={}",
            provider_url.host_str().unwrap(),
            provider_url.path().rsplit('/').next().unwrap()
        );

        let sources_json: Value = serde_json::from_str(get_response_sources(&url)?.as_str())?;


        let enc_video_url = sources_json["sources"].as_str().ok_or("Missing 'sources' field")?.to_string();
        let encrypted = sources_json["encrypted"].as_bool().ok_or("Missing 'encrypted' field")?;

        let mut sub_source = String::new();
        if let Some(english_sub) = sources_json["tracks"].as_array().ok_or("Missing 'tracks' field")?.iter().find(|v| v["label"].to_string().contains("English")) {
            sub_source = english_sub["file"].as_str().unwrap_or_default().to_string();
        }

        if !encrypted {
            println!("NOT ECRYPTED !!!!!!!!!!!!!!!!!!!!!!!!!!1");
            println!("{}", enc_video_url);
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
        &get_response(String::from("https://raw.githubusercontent.com/enimax-anime/key/e4/key.txt"))
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
        println!("{}Could't decrypt video source url", "\x1b[31m");
        std::process::exit(1)
    }


    decrypted_source
}


fn get_ep_data_id(ep_id: u32) -> u32 {
    let req = scraper::Html::parse_document(&get_response(format!("https://flixhq.to/ajax/v2/episode/servers/{}", ep_id)).unwrap());
    let a_sel = Selector::parse("a").unwrap();
    req.select(&a_sel).next().unwrap().value().attr("data-id").unwrap().parse::<u32>().unwrap() 
}

#[tokio::main] 
async fn get_response_sources(url: &str,) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    Ok(client
        .get(url)
        .header("X-Requested-With","XMLHttpRequest")
        .send().await?
        .text().await? 
        .to_string()
    )
}
