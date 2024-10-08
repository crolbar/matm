use crate::utils::{get_response, Sources};
use std::{process::Command, collections::HashMap};
use serde::{Deserialize, Serialize};
use crate::hist::{Hist, DataType};
use scraper::Selector;
use serde_json::Value;

mod mov_select;

pub fn search_movie_show(select_provider: bool, vlc: bool, autoplay: bool) -> std::io::Result<()> {
    let mut mov = Mov::select_movie_show(
        &get_query().replace(" ", "-")
    )?;
    if select_provider {
        if !mov.name.contains("(movie)") {
            mov.set_providers();
        }
        mov.select_provider()?
    }
    mov.vlc = vlc;
    mov.autoplay = autoplay;

    Ok(mov.main_loop()?)
}

pub fn select_from_hist(select_provider: bool, vlc: bool, autoplay: bool) -> std::io::Result<()> {
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

    mov.update_ep_ids();
    if select_provider {
        mov.set_providers();
        mov.select_provider()?
    }
    mov.vlc = vlc;
    mov.autoplay = autoplay;

    Ok(mov.main_loop()?)
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Mov {
    #[serde(skip)]
    pub ep_ids: Vec<String>,
    pub season_id: usize,
    pub name: String,
    pub ep: usize,
    #[serde(skip)]
    pub sel_provider: String,
    #[serde(skip)]
    pub providers: HashMap<String, String>,
    #[serde(skip)]
    pub vlc: bool,
    #[serde(skip)]
    pub autoplay: bool,
}

impl Mov {
    fn main_loop(mut self) -> std::io::Result<()> {
        let mut err_msg: Option<&str> = None;
        self.play();

        loop {
            self.save_to_hist();

            if self.autoplay {
                self.ep += 1;

                if self.ep > self.ep_ids.len() {
                    println!("{}Episode out of bound", "\x1b[31m");
                    std::process::exit(0) 
                } 

                self.play();
            } else {
                if self.name.contains("(movie)") {
                    let select = selector::select(
                        vec![String::from("replay"),
                            String::from("change provider"),
                            String::from("search"),
                            String::from("quit")
                        ], Some(&self.name), err_msg 
                    )?;

                    match select.as_str() {
                        "replay" => self.play(),
                        "search" => {
                            self = Mov::select_movie_show(&get_query().replace(" ", "-"))?;
                            self.play()
                        },
                        "change provider" => self.select_provider()?,
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
                            String::from("autoplay"),
                            String::from("search"),
                            String::from("quit")
                        ], Some(&format!("Current ep - {} of {}", self.ep, self.name)), err_msg 
                    )?;

                    match select.as_str() {
                        "play next" => {
                            self.ep += 1;

                            if self.ep > self.ep_ids.len() {
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
                                (1..=self.ep_ids.len()).map(|x| x.to_string()).collect(),
                                Some("select episode"), None
                            )?.parse().unwrap()
                        },
                        "change provider" => self.select_provider()?,
                        "autoplay" => self.autoplay = true,
                        "search" => {
                            self = Self::select_movie_show(&get_query().replace(" ", "-")).unwrap();
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
        }
    }

    fn play(&mut self) {
        if !self.name.contains("(movie)") {
            self.set_providers();
        }
        match get_sources(self.providers.get(&self.sel_provider).unwrap().to_owned()) {
            Ok(sources) => {
                let title = 
                    if self.name.contains("(movie)") {
                        self.name.split_once("(movie)").unwrap().0.to_string()
                    } else {
                        format!("{} Episode: {}",
                            self.name.split("(tv) ").collect::<String>(),
                            self.ep
                        )
                    };

                println!("{}Playing: {}", "\x1b[32m", title);
                if self.vlc {
                    Command::new("vlc")
                            .args([
                                sources.video,
                                format!("--meta-title={}", title)
                            ]).spawn().expect("crashed trying to start vlc")
                            .wait().unwrap();
                } else {

                    #[cfg(not(target_os = "android"))]
                    {
                        let args = 
                            if sources.subs.is_empty() {
                                println!("{}Could't find subtitles", "\x1b[31m");

                                vec![
                                    format!("{}", sources.video),
                                    format!("--force-media-title={}", title),
                                    String::from("--fs"),
                                ]
                            } else {
                                vec![
                                    format!("{}", sources.video),
                                    format!("--sub-file={}", sources.subs),
                                    format!("--force-media-title={}", title),
                                    String::from("--fs"),
                                ]
                            };

                        Command::new("mpv")
                            .args(args)
                            .spawn().expect("crashed trying to start mpv")
                            .wait().unwrap();
                    }

                    #[cfg(target_os = "android")]
                    {
                        let args = {
                            format!(
                                "am start -a android.intent.action.VIEW -n is.xyz.mpv/.MPVActivity -d \"{}\" >/dev/null",
                                sources.video 
                            )
                        };

                        Command::new("sh")
                            .arg("-c")
                            .arg(args)
                            .spawn().expect("crashed trying to start mpv")
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

    fn select_provider(&mut self) -> std::io::Result<()> {
        let mut p = self.providers.keys()
            .filter(|i| *i == "Vidcloud" || *i == "UpCloud")
            .map(|i| i.to_owned())
            .collect::<Vec<String>>();
        p.sort();

        self.sel_provider = 
            selector::select(
                p, Some("Change the provider server. (default one is probably upcloud)"), None
            )?;
        Ok(())
    }

    fn set_providers(&mut self) {
        let a_sel = Selector::parse("a").unwrap();
        let url = format!( "https://flixhq.to/ajax/v2/episode/servers/{}",
                self.ep_ids[self.ep - 1]);
        let response = get_response(&url).unwrap();
        let provider_page = scraper::Html::parse_document(&response);

        let ids: Vec<String> = 
            provider_page
                .select(&a_sel)
                .map(|x| 
                     x.value()
                     .attr("data-id").unwrap()
                     .to_string()
                ).collect();

        let names: Vec<String> =
            provider_page
                .select(&a_sel)
                .map(|i| 
                     i.select(&Selector::parse("span").unwrap())
                        .map(|i| i.text().collect::<String>()).collect()
                 ).collect();

        if self.sel_provider.is_empty() {
            self.sel_provider = names[0].to_owned();
        }

        self.providers = names.into_iter().zip(ids).collect();
    }
    
    fn save_to_hist(&self) {
        if !self.name.contains("(movie)") {
            match self.ep + 1 > self.ep_ids.len() {
                true => {
                    if Hist::deserialize().mov_data.iter().position(|x| x.name == self.name) != None {
                        Hist::remove(&self.name, DataType::MovData);
                    }
                },
                false => Hist::mov_save(self.clone())
            }
        }
    }

    fn update_ep_ids(&mut self) {
        let a_sel = Selector::parse("a").unwrap();
        let response = 
            if let Ok(resp) = get_response(
                &format!("https://flixhq.to/ajax/v2/season/episodes/{}", self.season_id)
            ) {
                resp
            } else {
                println!("{}No internet connection", "\x1b[33m");
                std::process::exit(0)
            };
        let episodes_page = scraper::Html::parse_document(&response);

        self.ep_ids = 
            episodes_page
            .select(&a_sel)
            .map(|x| x.value().attr("data-id").unwrap().to_string())
        .collect()
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

fn get_sources(data_id: String) -> Result<Sources, Box<dyn std::error::Error>> {
    let url = format!("https://flixhq.to/ajax/sources/{}", data_id);
    let provider: Value = serde_json::from_str(&get_response(&url)?)?;
    let provider_url = provider["link"].as_str().ok_or("Missing 'link' field")?;

    let url = format!("https://crolbar.xyz/api/rb/{}",
        provider_url.rsplit_once("/").unwrap().1.split_once("?").unwrap().0
    );
    let response = get_response(&url)?;
    
    let sources_json: Value = if let Ok(sources) = serde_json::from_str::<Value>(&response) {
        sources
    } else { 
        println!("{}Couldn't deserialize sources page. Maybe the provier server is down?", "\x1b[31m");
        std::process::exit(1)
    };


    let subs = 
        if let Some(english_sub) =
            sources_json["tracks"]
                .as_array()
                .ok_or("Missing 'tracks' field")?
                .iter().find(|v| v["label"].to_string().contains("English")) 
        {
            english_sub["file"].as_str().unwrap_or_default().to_string()
        } else { String::new() };


    let video = sources_json["sources"]
        .as_array().unwrap()
        .first().unwrap()
        .as_object().unwrap()
        ["file"].as_str().unwrap()
        .to_string();

    Ok(Sources {video, subs})
}
