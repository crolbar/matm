use serde::{Deserialize, Serialize};
use futures::{stream, StreamExt};
use crate::hist::{Hist, DataType};
use crate::utils::get_response;
use scraper::{Html, Selector};
use zip::write::FileOptions;
use std::io::Write;
use zip::ZipWriter;
use std::fs::File;
mod man_select;

pub fn search_manga() -> std::io::Result<()> {
    let mut query = String::new();
    while query.trim().is_empty() {
        println!("{}Search for manga: {}", "\x1b[34m", "\x1b[0m");
        std::io::stdin().read_line(&mut query).expect("reading stdin");
    }

    let mut man: Man = Man::select_manga(&query.replace(" ", "_"))?;
    
    Ok(man.main_loop()?)
}

pub fn select_from_hist() -> std::io::Result<()> {
    let hist = Hist::deserialize();
    let name = selector::select(
        hist.man_data
            .iter()
            .map(|x| format!("{} Chapter: {}", x.name, x.chapter))
            .collect(),
        None, None
    )?.split_once(" Chapter")
        .unwrap_or_else(|| {
            println!("{}Exiting...", "\x1b[33m");
            std::process::exit(0) 
        }).0.to_string();

    let mut man: Man = hist.man_data.iter().find(|m| m.name == name).unwrap().clone();

    man.get_all_chapters();
    Ok(man.main_loop()?)
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Man {
    pub all_chapters: Vec<f32>,
    pub chapter: f32,
    pub url_id: String,
    pub name: String,
}

impl Man {
    fn main_loop(&mut self) -> std::io::Result<()> {
        check_missing_dirs();

        loop {
            self.read();
            let current_chapter_index = self.all_chapters.iter().position(|x| x == &self.chapter).unwrap();

            self.save_to_hist();
            self.download_next_chapter(current_chapter_index);

            let select = selector::select(
                vec![String::from("next"),
                    String::from("reload"),
                    String::from("previous"),
                    String::from("select chapter"),
                    String::from("search"),
                    String::from("quit")
                ],
                Some(&format!("Current chapter - {} of {}", self.chapter, self.name)), None
            )?;

            match select.as_str() {
                "next" => {
                    if &self.chapter >= self.all_chapters.last().unwrap() { 
                        println!("{}Episode out of bound", "\x1b[31m"); 
                        std::process::exit(0) 
                    };

                    self.chapter = self.all_chapters[current_chapter_index + 1].clone();
                },
                "reload" => 
                    std::fs::remove_file(
                        dirs::home_dir().unwrap()
                            .join(format!(".cache/matm/{}-{}.cbz", self.name, self.chapter))
                    ).unwrap(),
                "previous" => {
                    if self.chapter <= 0.0 { 
                        println!("{}Episode out of bound", "\x1b[31m");
                        std::process::exit(0) 
                    }; 

                    self.chapter = self.all_chapters[current_chapter_index - 1].clone()
                },
                "select chapter" => {
                    self.chapter = selector::select(
                        self.all_chapters.iter().map(|x| x.to_string()).collect(),
                        None, None
                    )?.parse().unwrap();
                },
                "search" => { 
                    let mut query = String::new();
                    println!("{}Search for selfga: {}", "\x1b[34m", "\x1b[0m");
                    std::io::stdin().read_line(&mut query).expect("reading stdin");
                    *self = Man::select_manga(&query.replace(" ", "_"))?;
                },
                _ => std::process::exit(0)
            }
        }
    }

    fn read(&self) {
        self.create_cbz();

        std::process::Command::new("zathura")
            .args([
                dirs::home_dir().unwrap()
                    .join(
                        format!(".cache/matm/{}-{}.cbz", self.name, self.chapter)
                    ).to_str().unwrap(),
               "--mode=fullscreen"
            ])
        .spawn().unwrap();
    }


    fn create_cbz(&self) {
        let home_dir = dirs::home_dir().unwrap();

        if 
            std::fs::metadata(
                home_dir.join(
                    format!(".cache/matm/{}-{}.cbz", self.name, self.chapter)
                )
            ).is_err() 
        {
            println!("{}Downloading chapter {} of {}", "\x1b[32m", self.chapter, self.name);

            let response = get_response(
                &format!(
                    "https://chapmanganato.com/{}/chapter-{}",
                    self.url_id.rsplit_once("/").unwrap().1,
                    self.chapter
                )
            ).unwrap();
            let page = Html::parse_document(&response);

            let div_sel = Selector::parse("div.container-chapter-reader").unwrap();
            let img_sel = Selector::parse("img").unwrap();
            

            let img_urls = page.select(&div_sel).next().unwrap()
                .select(&img_sel)
                .map(|i| 
                    i.value().attr("src").unwrap().to_string()
                ).collect::<Vec<_>>();

            get_imgs(img_urls);
            
            std::fs::rename(
                home_dir.join(".cache/matm/false.cbz"),
                home_dir.join(format!(".cache/matm/{}-{}.cbz", self.name, self.chapter))
            ).unwrap()
        }
    }

    fn get_all_chapters(&mut self) {
        let response = get_response(&self.url_id)
            .unwrap_or_else(|_| {
                println!("{}No internet connection", "\x1b[33m");
                std::process::exit(0) 
            });
        
        let page = Html::parse_document(&response);

        let li_sel = Selector::parse("li.a-h").unwrap();
        let a_sel = Selector::parse("a").unwrap();

        let all_chapters: Vec<f32> = 
            page.select(&li_sel).rev().map(|c|
                    c.select(&a_sel)
                    .next().unwrap()
                    .value().attr("href").unwrap()
                    .rsplit_once('-').unwrap().1
                    .parse().unwrap()
            ).collect();

        if all_chapters.len() == 1 {
            println!("{}No chapters found", "\x1b[31m",);
            std::process::exit(0)
        }

        self.all_chapters = all_chapters;
    }

    fn download_next_chapter(&self, current_chapter_index: usize) {
        if &self.chapter < self.all_chapters.last().unwrap() {
            let mut tmp_self = self.clone();
            tmp_self.chapter = tmp_self.all_chapters[current_chapter_index + 1].clone();
            tmp_self.create_cbz();
        }
    }

    fn save_to_hist(&self) {
        match &self.chapter >= self.all_chapters.last().unwrap() {
            true => {
                if Hist::deserialize().man_data.iter().position(|x| x.name == self.name) != None {
                    Hist::remove(&self.name, DataType::ManData);
                }
            },
            false => Hist::man_save(self.clone())
        }
    }
}

#[tokio::main]
async fn get_imgs(img_urls: Vec<String>) {
    let client = reqwest::Client::new();

    let zip_writer = 
        std::sync::Arc::new(
            std::sync::Mutex::new(
                ZipWriter::new(
                    File::create(dirs::home_dir().unwrap().join(".cache/matm/false.cbz")).unwrap()
                )
            )
        );

    let imgs = stream::iter(img_urls).map(|url| {
        let client = client.clone();
            tokio::spawn(async move {
                client.get(url)
                    .header("Referer","https://readmanganato.com/")
                    .send().await.unwrap()
                    .bytes().await.unwrap()
            })
    });

    imgs.enumerate().for_each(|(n, img)|{
        let zip_writer = zip_writer.clone();
        async move {
            let mut zip_writer = zip_writer.lock().unwrap();
            
            zip_writer.start_file(
                format!("{}.jpg", n + 10),
                FileOptions::default().compression_method(zip::CompressionMethod::Stored)
            ).unwrap();
            
            zip_writer.write_all(&img.await.unwrap()).unwrap();
            println!("{}Downloaded image: {}{}","\x1b[90m", n, "\x1b[0m");
        }
    }).await
}


pub fn delete_cache() {
    std::fs::remove_dir_all(dirs::home_dir().unwrap().join(".cache/matm")).unwrap();
    std::fs::create_dir(dirs::home_dir().unwrap().join(".cache/matm")).unwrap();
    println!("{}Cache cleared", "\x1b[34m")
}

fn check_missing_dirs() {
    let home_dir = dirs::home_dir().unwrap();

    if std::fs::metadata(home_dir.join(".cache/matm")).is_err() { 
        std::fs::create_dir_all(home_dir.join(".cache/matm")).unwrap() 
    }

    if std::fs::metadata(home_dir.join(".cache/matm/false.cbz")).is_ok() {
        std::fs::remove_file(home_dir.join(".cache/matm/false.cbz")).unwrap() 
    }
}
