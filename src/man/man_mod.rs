use serde::{Deserialize, Serialize};
use futures::{stream, StreamExt};
use crate::utils::get_response;
use scraper::{Html, Selector};
use zip::write::FileOptions;
use std::io::Write;
use zip::ZipWriter;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Man {
    pub all_chapters: Vec<f32>,
    pub chapter: f32,
    pub url_id: String,
    pub name: String,
}

impl Man {
    pub fn read(&self) {
        self.create_cbz();


        std::process::Command::new("zathura")
            .args([
                dirs::home_dir().unwrap().join(format!(".cache/mani/{}-{}.cbz", self.name, self.chapter)).to_str().unwrap(),
               "--mode=fullscreen"
            ])
        .spawn().unwrap();
    }

    pub fn get_all_chapters(url_id: &str) -> Vec<f32> {
        let response = get_response(url_id).unwrap();
        let page = Html::parse_document(&response);

        let li_sel = Selector::parse("li.a-h").unwrap();
        let a_sel = Selector::parse("a").unwrap();

        let all_chapters: Vec<f32> = 
            page.select(&li_sel).rev().map(|c|
                (
                    c.select(&a_sel).next().unwrap().value().attr("href").unwrap().rsplit_once('-').unwrap().1.parse().unwrap()
                )
            ).collect();

        if all_chapters.len() == 1 {
            println!("{}No chapters found", "\x1b[31m",);
            std::process::exit(0)
        }

        all_chapters
    }

    pub fn create_cbz(&self) {
        if std::fs::metadata(dirs::home_dir().unwrap().join(format!(".cache/mani/{}-{}.cbz", self.name, self.chapter))).is_err() {
            println!("{}Downloading chapter {} of {}", "\x1b[32m", self.chapter, self.name);
            let response = get_response(&format!("https://chapmanganato.com/{}/chapter-{}", self.url_id.rsplit_once("/").unwrap().1, self.chapter)).unwrap();
            let page = Html::parse_document(&response);

            let div_sel = Selector::parse("div.container-chapter-reader").unwrap();
            let img_sel = Selector::parse("img").unwrap();
            

            let img_urls = page.select(&div_sel).next().unwrap()
                .select(&img_sel)
                .map(|i| 
                    i.value().attr("src").unwrap().to_string()
                ).collect::<Vec<_>>();

            
            get_imgs(img_urls);
            
            std::fs::rename(dirs::home_dir().unwrap().join(".cache/mani/false.cbz"), format!(".cache/mani/{}-{}.cbz", self.name, self.chapter)).unwrap();
        }
    }
}

#[tokio::main]
async fn get_imgs(img_urls: Vec<String>) {
    let client = reqwest::Client::new();

    let zip_writer = std::sync::Arc::new(std::sync::Mutex::new(
        ZipWriter::new(
            File::create(dirs::home_dir().unwrap().join(".cache/mani/false.cbz")).unwrap()
        )
    ));

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
            zip_writer.start_file(format!("{}.jpg", n + 10), FileOptions::default().compression_method(zip::CompressionMethod::Stored)).unwrap();
            
            zip_writer.write_all(&img.await.unwrap()).unwrap();
            println!("{}Downloaded image: {}{}","\x1b[90m", n, "\x1b[0m");
        }
    }).await
}