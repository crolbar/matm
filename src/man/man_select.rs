use crate::utils::get_response;
use scraper::{Html, Selector};
use crate::man::Man;

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

        let names: Vec<(String, String)> = 
            search_page.select(&div_sel).map(|i| {
                let a_elem = i.select(&h3_sel).next().unwrap().select(&a_sel).next().unwrap();

                (
                    a_elem.text().collect::<Vec<_>>().join(""), // name
                    a_elem.value().attr("href").unwrap().to_string() // url
                )
            }).collect();


        if names.is_empty() {
            println!("{}No results found", "\x1b[31m");
            std::process::exit(0)
        }

        let name = selector::select(
            names.iter().map(|x| x.0.clone()).collect(),
            None, None
        )?;
        if name.is_empty() { std::process::exit(0) }


        let (url_id, name) = {
            let n = names.iter().find(|i| *i.0 == name).unwrap();

            (n.1.to_string(), n.0.to_string())
        };

        let mut man = Self { 
            all_chapters: vec![],
            chapter: 0.0,
            url_id,
            name,
        };
        man.get_all_chapters();

        man.chapter = selector::select(
            man.all_chapters.clone().iter().map(|x| x.to_string()).collect(),
            None, None
        )?.parse().unwrap_or_else(|_| std::process::exit(0));

        Ok(man)
    }
}
