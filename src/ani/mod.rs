use crate::utils::{get_response, decrypt_url, Sources, extract_key};
use std::{process::Command, collections::HashMap};
use serde::{Deserialize, Serialize};
use crate::hist::{Hist, DataType};
use scraper::{Html, Selector};
use serde_json::Value;
mod ani_select;

pub fn search_anime(select_provider: bool, is_dub: bool) -> std::io::Result<()> {
    let mut ani = Ani::select_anime(&get_query())?;

    if select_provider {
        ani.set_providers();
        ani.select_provider()?
    }
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

    ani.update_ep_ids();
    if select_provider {
        ani.set_providers();
        ani.select_provider()?
    }
    ani.is_dub = is_dub;

    Ok(ani.main_loop()?)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Ani {
    #[serde(skip)]
    pub ep_ids: Vec<u32>,
    pub name: String,
    pub ep: usize,
    pub id: usize,
    #[serde(skip)]
    pub sel_provider: String,
    #[serde(skip)]
    pub providers: HashMap<String, String>,
    #[serde(skip)]
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
                    
                    if self.ep > self.ep_ids.len() {
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
                        (1..=self.ep_ids.len()).map(|x| x.to_string()).collect(),
                        None, None
                    )?.parse()
                        .unwrap_or_else(|_| {
                            println!("{}Exiting...", "\x1b[33m");
                            std::process::exit(0) 
                        })
                },
                "switch to sub" => self.is_dub = false,
                "switch to dub" => self.is_dub = true,
                "change provider" => self.select_provider()?,
                "search" => {
                    *self = Ani::select_anime(&get_query())?;
                    self.play()
                },
                "quit" => std::process::exit(0),
                _ => ()
            }

            if 
                self.ep > self.ep_ids.len() ||
                self.ep == 0
            {
                err_msg = Some("Episode out of bound");
            } else { err_msg = None }
        }
    }

    fn play(&mut self) {
        self.set_providers();
        match get_sources(&self.providers.get(&self.sel_provider).unwrap()) {
            Ok(sources) => {
                println!("{}Playing: {} Ep: {}", "\x1b[32m", self.name, self.ep);
                let args = 
                    if sources.subs.is_empty() {
                        println!("{}Could't find subtitles", "\x1b[31m");

                        vec![
                            sources.video,
                            format!("--force-media-title={} Episode {}", self.name, self.ep),
                            String::from("--fs")
                        ]
                    } else {
                        vec![
                            sources.video,
                            format!("--sub-file={}",sources.subs),
                            format!("--force-media-title={} Episode {}", self.name, self.ep),
                            String::from("--fs")
                        ]
                    };

                Command::new("mpv")
                    .args(args)
                    .spawn().expect("crashed when trying to start mpv")
                    .wait().unwrap();
            }
            Err(e) => {
                println!("{}Error while trying to get sources: {}", "\x1b[31m", e);
                std::process::exit(1) 
            }
        }
    }

    fn select_provider(&mut self) -> std::io::Result<()> {
        self.sel_provider = 
                selector::select(
                    self.providers.keys().map(|n|n.to_owned()).collect(),
                    Some("Change the provider server. (supported ones: Vidstreaming, MegaCloud)"), None
                )?;
        if self.sel_provider.is_empty() {
            println!("{}Exiting...", "\x1b[33m");
            std::process::exit(0) 
        }
        Ok(())
    }

    fn set_providers(&mut self) {
        let ep_id = self.ep_ids[self.ep.clone() - 1];

        let video_type = if self.is_dub {"dub"} else {"sub"};

        let response = get_response(
            &format!("https://aniwatch.to/ajax/v2/episode/servers?episodeId={}", ep_id)
        ).unwrap();

        let mut pattern = format!("data-type=\\\"{}\\\" data-id=\\\"", video_type);
        
        if !response.contains(&pattern) && self.is_dub {
            println!("{}This episode doesn't have an dubbed version. Playing with subs", "\x1b[31m");
            pattern = pattern.replace("dub", "sub");
            self.is_dub = false;
        }

        if !response.contains(&pattern) {
            let video_type = if self.is_dub {"dub"} else {"sub"};
            pattern = pattern.replace(video_type, "raw");
        }

        let provider_info = response.split(&pattern).skip(1);

        let ids: Vec<String> = 
            provider_info.clone()
            .map(|x| x.split_once("\\\"\\n").unwrap().0.to_string())
            .collect();

        let names: Vec<String> =
            provider_info
            .map(|x| x
                 .split_once("</a>").unwrap().0.to_string()
                 .rsplit_once(">").unwrap().1.to_string()
            ).collect();

        if self.sel_provider.is_empty() {
            self.sel_provider = names[0].to_owned();
        }

        self.providers = names.into_iter().zip(ids).collect();
    }

    fn save_to_hist(&self) {
        match self.ep + 1 > self.ep_ids.len() {
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
            episodes_html.select(&ep_sel)
                .map(|x| x.value()
                     .attr("data-id").unwrap()
                     .parse::<u32>().unwrap()
                ).collect()
    }
}

fn get_query() -> String {
    let mut query = String::new();
    while query.trim().is_empty() {
        println!("{}Search for anime: {}", "\x1b[34m", "\x1b[0m");
        std::io::stdin().read_line(&mut query).expect("reading stdin");
    }
    query
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


    let mut subs = String::new();
    if let Some(english_sub) = 
        sources_json["tracks"]
            .as_array()
            .ok_or("Missing 'tracks' field")?
            .iter().find(|v| v["label"] == "English") 
    {
        subs = english_sub["file"].as_str().unwrap_or_default().to_string();
    };

    let video = 
        if sources_json["encrypted"].as_bool().unwrap() {
            let enc_video_url = sources_json["sources"].as_str().unwrap().to_string();

            let key: Vec<Vec<u32>> = {
                let (url, fallback_url) = {
                    let e = provider_url.path().split_once("e-").unwrap().1.chars().next().unwrap();

                    (
                        format!( "http://crolbar.xyz/key/e{}", e),
                        format!( "https://raw.githubusercontent.com/AuraStar553/keys/e{}/key", e)
                    )
                };

                if let Ok(key) = &get_response(&url) {
                    serde_json::from_str(key).expect("couldnt deserialize string to vec")
                } else {
                    let key: Vec<Vec<u32>> = serde_json::from_str(
                        &get_response(&fallback_url).expect("couldn't get key")
                    ).expect("couldnt deserialize string to vec");

                    let mut sum = 0;
                    let mut key = key;
                    let init_key = key.clone();
                    for (i, _) in init_key.iter().enumerate() {
                        key[i][0] = init_key[i][0] + sum;
                        sum += init_key[i][1];
                        key[i][1] = init_key[i][0] + sum;
                    }  

                    key
                }
            };

            let url_key = extract_key(enc_video_url, key);
            decrypt_url(url_key.0, url_key.1)
        } else { 
            sources_json["sources"]
                .as_array().unwrap()
                [0].as_object().unwrap()
                ["file"].as_str().unwrap()
                .to_string() 
        };

    Ok(Sources{video, subs})
}
