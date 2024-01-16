use crate::utils::{get_response, decrypt_url, Sources};
use serde::{Deserialize, Serialize};
use crate::hist::{Hist, DataType};
use scraper::{Html, Selector};
use ani_select::select_anime;
use std::process::Command;
use serde_json::Value;
mod ani_select;

pub fn search_anime(select_provider: bool, is_dub: bool) -> std::io::Result<()> {
    let mut query = String::new();

    while query.trim().is_empty() {
        println!("{}Search for anime: {}", "\x1b[34m", "\x1b[0m");
        std::io::stdin().read_line(&mut query).expect("reading stdin");
    }

    let mut ani = select_anime(&query)?;
    ani.get_provider_index(select_provider)?;
    ani.is_dub = is_dub;

    Ok(ani.main_loop()?)
}

pub fn select_from_hist(select_provider: bool, is_dub: bool) -> std::io::Result<()> {
    let hist = Hist::deserialize();

    let name = 
        selector::select(
            hist.ani_data
            .iter()
            .map(|x| format!("{} Episode: {}", x.name, x.ep))
            .collect(),
            None, None
            )?.split_once(" Episode")
        .unwrap_or_else(|| {
            println!("{}Exiting...", "\x1b[33m");
            std::process::exit(0) 
        }).0.to_string();

    let mut ani = hist.ani_data.iter().find(|x| x.name == name).unwrap().clone();
    ani.get_provider_index(select_provider)?;
    ani.is_dub = is_dub;

    ani.update_ep_ids();

    Ok(ani.main_loop()?)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Ani {
    pub ep_ids: Option<Vec<u32>>,
    pub name: String,
    pub ep: usize,
    pub id: usize,
    pub provider_index: usize,
    pub is_dub: bool,
}

impl Ani {
    fn main_loop(&mut self) -> std::io::Result<()> {
        let mut err_msg: Option<&str> = None;
        self.play();

        loop {
            self.save_to_hist();
            
            let select = selector::select(
                vec![String::from("play next"),
                    String::from("play"),
                    String::from("next"),
                    String::from("previous"),
                    String::from("select ep"),
                    format!("switch to {}", if self.is_dub {"sub"} else {"dub"}),
                    String::from("change provider"),
                    String::from("search"),
                    String::from("quit")
                ], 
                Some(&format!("Current ep - {} of {}", self.ep, self.name)), err_msg
            )?;

            match select.as_str() {
                "play next" => {
                    self.ep += 1;
                    
                    if self.ep > self.ep_ids.clone().unwrap().len() {
                        println!("{}Episode out of bound", "\x1b[31m");
                        std::process::exit(0) 
                    }

                    self.play();
                },
                "play" => self.play(),
                "next" => self.ep += 1,
                "previous" => self.ep = self.ep.saturating_sub(1),
                "select ep" => {
                    self.ep = selector::select(
                        (1..=self.ep_ids.clone().unwrap().len()).map(|x| x.to_string()).collect(),
                        None, None
                    )?.parse()
                        .unwrap_or_else(|_| {
                            println!("{}Exiting...", "\x1b[33m");
                            std::process::exit(0) 
                        })
                },
                "switch to sub" => self.is_dub = false,
                "switch to dub" => self.is_dub = true,
                "change provider"  => self.get_provider_index(true)?,
                "search" => {
                    let mut query = String::new();
                    println!("{}Search for selfme: {}", "\x1b[34m", "\x1b[0m");
                    std::io::stdin().read_line(&mut query).expect("reading stdin");
                    *self = select_anime(&query)?
                },
                "quit" => std::process::exit(0),
                _ => ()
            }

            if 
                self.ep > self.ep_ids.clone().unwrap().len() ||
                self.ep == 0
            {
                err_msg = Some("Episode out of bound");
            } else { err_msg = None }
        }
    }

    fn play(&mut self) {
        let mut data_ids = self.get_ep_data_ids();

        if self.provider_index >= data_ids.len() {
            self.is_dub = false;
            println!("{}This episode dosnt have an dubbed version", "\x1b[31m");
            data_ids = self.get_ep_data_ids()
        }

        match get_sources(&data_ids[self.provider_index]) {
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

    fn get_provider_index(&mut self, select_provider: bool) -> std::io::Result<()> {
        self.provider_index = 
            if select_provider {
                    selector::select(
                        (1..=self.get_ep_data_ids().len()).map(|x| x.to_string()).collect(),
                        Some(
                            "
                                Change the provider server.
                                (usualy the last ones are not supported)
                                (if you havent changed it, it defaults to the first)
                            "
                        ), None
                    )?
                    .parse::<usize>().unwrap_or_else(|_| {
                        println!("{}Exiting...", "\x1b[33m");
                        std::process::exit(0) 
                    }) - 1
            } else { 0 };
        Ok(())
    }

    fn get_ep_data_ids(&self) -> Vec<String> {
        let ep_id = self.ep_ids.clone().unwrap()[self.ep.clone() - 1];

        let video_type = if self.is_dub {"dub"} else {"sub"};

        let response = get_response(
            &format!("https://aniwatch.to/ajax/v2/episode/servers?episodeId={}", ep_id)
        ).unwrap();

        let provider_list: Vec<String> =
            response.split(format!("data-type=\\\"{}\\\" data-id=\\\"", video_type).as_str()).skip(1)
                .map(|x| x.split_once("\\\"\\n").unwrap().0.to_string())
                .collect();

        if provider_list.is_empty() {
            response.split(format!("data-type=\\\"{}\\\" data-id=\\\"", "raw").as_str()).skip(1)
                .map(|x| x.split_once("\\\"\\n").unwrap().0.to_string())
                .collect()
        } else {
            provider_list
        }
    }

    fn save_to_hist(&self) {
        match self.ep + 1 > self.ep_ids.clone().unwrap().len() {
            true => {
                if let Some(_) = Hist::deserialize().ani_data.iter().position(|x| x.name == self.name)  {
                    Hist::remove(&self.name, DataType::AniData);
                }
            },
            false => Hist::ani_save(self.clone())
        }
    }

    fn update_ep_ids(&mut self) {
        let episodes_url = format!("https://aniwatch.to/ajax/v2/episode/list/{}", self.id);
        let episodes_json: Value = 
            serde_json::from_str(
                &get_response(&episodes_url).unwrap_or_else(|_| {
                    println!("{}No internet connection", "\x1b[33m");
                    std::process::exit(0) 
                })
            ).unwrap();

        let episodes_html = Html::parse_document(episodes_json["html"].as_str().unwrap());
        let ep_sel = Selector::parse("a.ssl-item").unwrap();

        self.ep_ids = 
            Some(
                episodes_html.select(&ep_sel)
                    .map(|x| x.value()
                         .attr("data-id").unwrap()
                         .parse::<u32>().unwrap()
                    ).collect()
            )
    }
}


fn get_sources(data_id: &str) -> Result<Sources, Box<dyn std::error::Error>> {
    let url = format!("https://aniwatch.to/ajax/v2/episode/sources?id={}", data_id);
    let provider: Value = serde_json::from_str(&get_response(&url)?)?;
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
    if let Some(english_sub) = 
        sources_json["tracks"]
            .as_array()
            .ok_or("Missing 'tracks' field")?
            .iter().find(|v| v["label"] == "English") 
    {
        sub_source = english_sub["file"].as_str().unwrap_or_default().to_string();
    };

    let video_source = 
        if sources_json["encrypted"].as_bool().unwrap() {
            let enc_video_url = sources_json["sources"].as_str().unwrap().to_string();

            let url = 
                format!(
                    "http://crolbar.xyz/key/e{}",
                    provider_url.path().split_once("e-").unwrap().1.chars().next().unwrap()
                );

            let key: Vec<Vec<u32>> = serde_json::from_str(&get_response(&url)
                .expect("couldnt get key")).expect("couldnt deserialize string to vec");

            decrypt_url(enc_video_url, key)
        } else { 
            sources_json["sources"]
                .as_array().unwrap()
                [0].as_object().unwrap()
                ["file"].as_str().unwrap()
                .to_string() 
        };

    Ok(
        Sources {
            video: video_source,
            subs: sub_source,
        }
    )
}
