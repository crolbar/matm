use crate::utils::{get_sources_response, get_response, decrypt_url, Sources};
use serde::{Deserialize, Serialize};
use crate::hist::{Hist, DataType};
use std::process::Command;
use scraper::Selector;
use serde_json::Value;

mod mov_select;

pub fn search_movie_show(select_provider: bool, vlc: bool) -> std::io::Result<()> {
    let mut mov = Mov::select_movie_show(
        &get_query().replace(" ", "-")
    )?;
    match select_provider {
        true => mov.set_provider_index()?,
        false => mov.provider_index = 0
    }
    mov.vlc = vlc;

    Ok(mov.main_loop()?)
}

pub fn select_from_hist(select_provider: bool, vlc: bool) -> std::io::Result<()> {
    let hist = Hist::deserialize();

    let name = 
        selector::select(
            hist.mov_data
                .iter()
                .map(|x| {
                    format!("{} Episode {}", x.name, x.ep)
            }).collect(),
            None, None
        )?.split_once(" Episode")
            .unwrap_or_else(|| {
                println!("{}Exiting...", "\x1b[33m");
                std::process::exit(0) 
            }).0
        .to_string();

    let mut mov = hist.mov_data.iter().find(|m| m.name == name).unwrap().clone();

    match select_provider {
        true => mov.set_provider_index()?,
        false => mov.provider_index = 0
    }
    mov.vlc = vlc;
    mov.update_ep_ids();

    Ok(mov.main_loop()?)
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Mov {
    pub ep_ids: Option<Vec<String>>,
    pub season_id: Option<usize>,
    pub provider_index: usize,
    pub name: String,
    pub ep: usize,
    pub vlc: bool,
}

impl Mov {
    fn main_loop(mut self) -> std::io::Result<()> {
        let mut err_msg: Option<&str> = None;
        self.play();

        loop {
            self.save_to_hist();

            if self.name.contains("(movie)") {
                let select = selector::select(
                    vec![String::from("search"),
                        String::from("replay"),
                        String::from("change provider"),
                        String::from("quit")
                    ], Some(&self.name), err_msg 
                )?;

                match select.as_str() {
                    "replay" => self.play(),
                    "search" => {
                        self = Mov::select_movie_show(&get_query().replace(" ", "-"))?;
                        self.play()
                    },
                    "change provider" => self.set_provider_index()?,
                    "quit" => std::process::exit(0),
                    _ => ()
                }
            } else {
                let select = selector::select(
                    vec![String::from("play next"),
                        String::from("play"),
                        String::from("next"),
                        String::from("previous"),
                        String::from("select ep"),
                        String::from("change provider"),
                        String::from("search"),
                        String::from("quit")
                    ], Some(&format!("Current ep - {} of {}", self.ep, self.name)), err_msg 
                )?;

                match select.as_str() {
                    "play next" => {
                        self.ep += 1;

                        if self.ep > self.ep_ids.clone().unwrap().len() {
                            println!("{}Episode out of bound", "\x1b[31m");
                            std::process::exit(0) 
                        } 

                        self.play();
                    }
                    "play" => self.play(),
                    "next" => self.ep += 1,
                    "previous" => self.ep -= 1,
                    "select ep" => {
                        self.ep = selector::select(
                            (1..=self.ep_ids.clone().unwrap().len()).map(|x| x.to_string()).collect(),
                            Some("select episode"), None
                        )?.parse().unwrap()
                    },
                    "change provider" => self.set_provider_index()?,
                    "search" => {
                        self = Self::select_movie_show(&get_query().replace(" ", "-")).unwrap();
                        self.play()
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
    }

    fn play(&mut self) {
        let url = 
            match self.name.contains("movie") {
                true => 
                    format!(
                        "https://flixhq.to/ajax/sources/{}",
                        self.ep_ids.clone().unwrap()[self.provider_index]
                    ),
                false => 
                    format!(
                        "https://flixhq.to/ajax/sources/{}",
                        self.get_ep_data_id()[self.provider_index]
                    )
            };
        match get_sources(url) {

            Ok(sources) => {
                println!("{}Playing: {} Episode: {}", "\x1b[32m", self.name, self.ep);

                let title = 
                    if self.name.contains("(movie)") {
                        self.name.split_once("(movie)").unwrap().0.to_string()
                    } else {
                        self.name.split("(tv) ").collect::<String>()
                    };

                if self.vlc {
                    Command::new("vlc")
                            .args([
                                sources.video,
                                format!("--meta-title={}", title)
                            ]).spawn().expect("crashed trying to start vlc")
                            .wait().unwrap();
                } else {
                    let args = 
                        if sources.subs.is_empty() {
                            println!("{}Could't find subtitles", "\x1b[31m");

                            vec![
                                sources.video,
                                format!("--force-media-title={}", title),
                                String::from("--fs")
                            ]
                        } else {
                            vec![
                                sources.video,
                                format!("--sub-file={}",sources.subs),
                                format!("--force-media-title={}", title),
                                String::from("--fs")
                            ]
                        };

                    Command::new("mpv")
                        .args(args)
                        .spawn().expect("crashed trying to start mpv")
                        .wait().unwrap();
                }
            }
            Err(e) => {
                println!("{}Error while trying to get sources: {}", "\x2b[31m", e);
                std::process::exit(1) 
            }
        }
    }

    fn set_provider_index(&mut self) -> std::io::Result<()> {
        let range: Vec<String> = match self.name.contains("(movie)") {
            true => 1..=self.ep_ids.clone().unwrap().len(),
            false => 1..=self.get_ep_data_id().len()
        }.map(|x| x.to_string()).collect();

        self.provider_index = 
            if range.len() > 1 {
                selector::select(
                    range,
                    Some(
                        "
                            Change the provider server.
                            (usualy the last ones are not supported)
                            (if you havent changed it, it defaults to the first)
                        "
                    ), None
                ).unwrap()
                .parse::<usize>().unwrap_or_else(|_| {
                    println!("{}Exiting...", "\x1b[33m");
                    std::process::exit(0) 
                }) - 1
            } else { 0 };
        Ok(())
    }

    fn update_ep_ids(&mut self) {
        let a_sel = Selector::parse("a").unwrap();
        let response = 
            if let Ok(resp) = get_response(
                &format!("https://flixhq.to/ajax/v2/season/episodes/{}", self.season_id.unwrap())
            ) {
                resp
            } else {
                println!("{}No internet connection", "\x1b[33m");
                std::process::exit(0)
            };
        let episodes_page = scraper::Html::parse_document(&response);

        self.ep_ids = Some(
            episodes_page
            .select(&a_sel)
            .map(|x| x.value().attr("data-id").unwrap().to_string())
        .collect())
    }

    // these are the providers for the episode. self.ep_ids are the ids of all episodes
    fn get_ep_data_id(&self) -> Vec<String> {
        let a_sel = Selector::parse("a").unwrap();
        let response = get_response(
            &format!( "https://flixhq.to/ajax/v2/episode/servers/{}",
                self.ep_ids.clone().unwrap()[self.ep - 1])
        ).unwrap();
        let provider_page = scraper::Html::parse_document(&response);

        provider_page.select(&a_sel).map(|x| x.value().attr("data-id").unwrap().to_string()).collect()
    }

    fn save_to_hist(&self) {
        if !self.name.contains("(movie)") {
            match self.ep + 1 > self.ep_ids.clone().unwrap().len() {
                true => {
                    if Hist::deserialize().mov_data.iter().position(|x| x.name == self.name) != None {
                        Hist::remove(&self.name, DataType::MovData);
                    }
                },
                false => Hist::mov_save(self.clone())
            }
        }
    }
}

fn get_query() -> String {
    let mut query = String::new();
    while query.trim().is_empty() {
        println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
        std::io::stdin().read_line(&mut query).expect("reading stdin");
    }
    query
}

fn get_sources(url: String) -> Result<Sources, Box<dyn std::error::Error>> {
    let provider: Value = serde_json::from_str(&get_response(&url)?)?;
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

    let mut subs = String::new();
    if let Some(english_sub) =
        sources_json["tracks"]
            .as_array()
            .ok_or("Missing 'tracks' field")?
            .iter().find(|v| v["label"].to_string().contains("English")) 
    {
        subs = english_sub["file"].as_str().unwrap_or_default().to_string()
    }

    let video = if sources_json["encrypted"].as_bool().unwrap() {
        let enc_video_url = sources_json["sources"].as_str().unwrap().to_string();

        let (url, fallback_url) = {
            let e = provider_url.path().split_once("embed-").unwrap().1.chars().next().unwrap();

            (
                format!( "http://crolbar.xyz/key/e{}", e),
                format!( "http://zoro-keys.freeddns.org/keys/e{}/key.txt", e)
            )
        };

        let key: Vec<Vec<u32>> = serde_json::from_str(
            &get_response(&url)
            .unwrap_or(
                get_response(&fallback_url).expect("couldn't get key")
            )).expect("couldnt deserialize vec");

        decrypt_url(enc_video_url, key)
    } else { 
        sources_json["sources"]
            .as_array().unwrap()[0]
            .as_object().unwrap()
            ["file"].as_str().unwrap()
            .to_string() 
    };

    Ok(Sources {video, subs})
}
