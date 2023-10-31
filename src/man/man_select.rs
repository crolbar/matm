use crate::man::man_mod::Man;
use crate::utils::get_response;
use scraper::{Html, Selector};

pub fn select_manga(query: &str) -> Man {
    let url = format!("https://manganato.com/search/story/{}", query);
    let response = get_response(&url).unwrap_or_else(|_| { println!("{}No internet connection", "\x1b[33m"); std::process::exit(0) });

    let div_sel = Selector::parse("div.search-story-item").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let a_sel = Selector::parse("a").unwrap();

    let search_page = Html::parse_document(&response);

    let mut search_results: Vec<(String, String)> = Vec::new();
    for i in search_page.select(&div_sel) {
        let a_elem = i.select(&h3_sel).next().unwrap().select(&a_sel).next().unwrap();
        search_results.push((
            a_elem.text().collect::<Vec<_>>().join(""),
            a_elem.value().attr("href").unwrap().to_string()
        ));
    }

    if search_results.is_empty() {
        println!("{}No results found", "\x1b[31m");
        std::process::exit(0)
    }

    let selected_name = rust_fzf::select(search_results.iter().map(|x| x.0.clone()).collect(), vec!["--reverse".to_string()]);
    if selected_name.is_empty() { std::process::exit(0) }

    let man = search_results.iter().find(|i| i.0 == selected_name).cloned().unwrap();
    let mut man = Man { all_chapters: Man::get_all_chapters(&man.1), chapter: 0.001, url_id: man.1, name: man.0 };

    man.chapter = rust_fzf::select(man.all_chapters.clone().iter().map(|x| x.to_string()).collect(), vec!["--reverse".to_string()]).parse().unwrap();
    if man.chapter == 0.001 { std::process::exit(1) }

    man
}
