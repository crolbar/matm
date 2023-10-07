use crate::utils::get_response;
use zip::{write::FileOptions, ZipWriter};
use scraper::{Html, Selector};
use std::io::Write;
use std::fs::File;

#[derive(Debug, Clone)]
pub struct Man {
    name: String,
    id: String,
    chapter: String
}

impl Man {
    pub fn select_chapter(&mut self) {
        if std::fs::metadata("/tmp/mani").is_err() { std::fs::create_dir_all("/tmp/mani/imgs").unwrap() }
        if std::fs::read_dir("/tmp/mani/imgs").unwrap().count() > 0 {std::fs::remove_dir_all("/tmp/mani/imgs").unwrap(); std::fs::create_dir("/tmp/mani/imgs").unwrap()}
        let response = get_response(&self.id).unwrap();
        let page = Html::parse_document(&response);

        let li_sel = Selector::parse("li.a-h").unwrap();
        let a_sel = Selector::parse("a").unwrap();

        let mut all_chapters: Vec<String> = vec![String::from("Download all chapters")];
        for i in page.select(&li_sel) {
            all_chapters.push(i.select(&a_sel).next().unwrap().text().collect::<Vec<_>>().join(""))
        }

        if all_chapters.len() == 1 {
            println!("{}No chapters found", "\x1b[31m");
            std::process::exit(0)
        }

        let chapter = rust_fzf::select(all_chapters.clone(), vec!["--reverse".to_string()]);
        if chapter.is_empty(){std::process::exit(0)}

        if chapter == String::from("Download all chapters") {
            for i in all_chapters.iter().skip(1) {
                    self.chapter = i.split_once("Chapter ").unwrap().1
                        .split_once(":").unwrap_or(
                            (i.split_once("Chapter ").unwrap().1, "")).0.to_string();
                    self.create_cbz()
                }
        } else {
            self.chapter = chapter.split_once("Chapter ").unwrap().1
                .split_once(":").unwrap_or(
                    (chapter.split_once("Chapter ").unwrap().1, "")).0.to_string()
        }
    }

    pub fn read(&self) {
        self.create_cbz();


        std::process::Command::new("zathura")
            .arg(format!("/tmp/mani/{}-{}.cbz", self.name, self.chapter)).spawn().unwrap();
    }

    fn create_cbz(&self) {
        if std::fs::metadata(format!("/tmp/mani/{}-{}.cbz", self.name, self.chapter)).is_err() {
            println!("{}Downloading chapter {} of {}", "\x1b[32m", self.chapter, self.name);
            let response = get_response(&format!("{}/chapter-{}", self.id, self.chapter)).unwrap();
            let page = Html::parse_document(&response);

            let div_sel = Selector::parse("div.container-chapter-reader").unwrap();
            let img_sel = Selector::parse("img").unwrap();
            
            let mut n = 11;
            for i in page.select(&div_sel).next().unwrap().select(&img_sel) {
                let url = i.value().attr("src").unwrap();
                get_image(url, n).unwrap();
                n += 1;
            }

            zip_files(&self.name, &self.chapter).unwrap();
        }
    }
}


pub fn select_manga(query: &str) -> Man {
    let url = format!("https://manganato.com/search/story/{}", query);
    let response = get_response(&url).unwrap();

    let div_sel = Selector::parse("div.search-story-item").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let a_sel = Selector::parse("a").unwrap();

    let search_page = Html::parse_document(&response);

    let mut search_results: Vec<Man> = Vec::new();
    let mut search_names: Vec<String> = Vec::new();

    for i in search_page.select(&div_sel) {
        let name = i.select(&h3_sel).next().unwrap().select(&a_sel).next().unwrap().text().collect::<Vec<_>>().join("");
        search_results.push( Man {
            name: name.clone(),
            id: i.select(&h3_sel).next().unwrap().select(&a_sel).next().unwrap().value().attr("href").unwrap().to_string(),
            chapter: "".to_string()
            }
        );
        search_names.push(name);
    }


    if search_names.is_empty() {
        println!("{}No results found", "\x1b[31m");
        std::process::exit(0)
    }

    let selected_name = rust_fzf::select(search_names, vec!["--reverse".to_string()]);

    if selected_name.is_empty() {std::process::exit(0)}

    dbg!(&search_results[1].id);

    search_results.iter().find(|i| i.name == selected_name).cloned().unwrap()
}


fn zip_files(name: &str, chapter: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cbz_file = File::create(format!("/tmp/mani/{}-{}.cbz", name, chapter))?;
    let mut zip_writer = ZipWriter::new(cbz_file);
    let images = std::fs::read_dir("/tmp/mani/imgs")?;


    let options = FileOptions::default().unix_permissions(0o755); // Set file permissions

    for image in images {
        let file_path = image?.path();

        if file_path.extension().unwrap() == "jpg" {
            zip_writer.start_file(file_path.file_name().unwrap().to_str().unwrap(), options)?;
            let mut jpg_file = std::fs::File::open(&file_path)?;
            std::io::copy(&mut jpg_file, &mut zip_writer)?;
            std::fs::remove_file(file_path)?;
        }
    }

    Ok(())
}

#[tokio::main] 
async fn get_image(url: &str, n: u32) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Referer", "https://readmanganato.com/")
        .send()
        .await?
        .bytes()
        .await?;

    std::fs::File::create(format!("/tmp/mani/imgs/{}.jpg", n)).unwrap()
        .write_all(&response).unwrap();

    Ok(())
}