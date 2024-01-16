use std::collections::HashMap;
use crate::man::Man;
use crate::utils::get_response;
use scraper::{Html, Selector};

impl Man {
    pub fn select_manga(query: &str) -> std::io::Result<Man> {
        let url = format!("https://manganato.com/search/story/{}", query);
        let response = get_response(&url)
            .unwrap_or_else(|_| {
                println!("{}No internet connection", "\x1b[33m");
                std::process::exit(0) 
            });

        let div_sel = Selector::parse("div.search-story-item").unwrap();
        let h3_sel = Selector::parse("h3").unwrap();
        let a_sel = Selector::parse("a").unwrap();

        let search_page = Html::parse_document(&response);

        let name_url_map: HashMap<String, String> = 
            search_page.select(&div_sel).map(|i| {
                let a_elem = i.select(&h3_sel).next().unwrap().select(&a_sel).next().unwrap();

                (
                    a_elem.text().collect::<Vec<_>>().join(""), // name
                    a_elem.value().attr("href").unwrap().to_string() // url
                )
            }).collect();


        if name_url_map.is_empty() {
            println!("{}No results found", "\x1b[31m");
            std::process::exit(0)
        }

        let name = selector::select(
            name_url_map.iter().map(|x| x.0.clone()).collect(),
            None, None
        )?;
        if name.is_empty() { std::process::exit(0) }


        let name_url = name_url_map.get_key_value(&name).unwrap();

        let mut man = Self { 
            all_chapters: vec![],
            url_id: name_url.1.to_string(),
            name: name_url.0.to_string(),
            chapter: 0.0,
        };
        man.get_all_chapters();

        man.chapter = selector::select(
            man.all_chapters.clone().iter().map(|x| x.to_string()).collect(),
            None, None
        )?.parse().unwrap_or_else(|_| std::process::exit(0));

        Ok(man)
    }
}
