use crate::utils::get_response;
use crate::ani::ani_mod::Ani;
use scraper::{Html, Selector};
use serde_json::Value;


pub fn select_anime(query: &str) -> Ani {
    let response = get_response(&format!("https://aniwatch.to/search?keyword={}", query)).unwrap();

    let div_sel = Selector::parse("div.film_list-wrap").unwrap();
    let h3_sel = Selector::parse("h3.film-name").unwrap();
    let link_sel = Selector::parse("a.dynamic-name").unwrap();

    let search_page = Html::parse_document(response.as_str());
    let search_results = search_page.select(&div_sel);

    let mut name_search_results:Vec<String> = Vec::new();
    let mut anime_ids:Vec<&str> = Vec::new();

    for result in search_results {
        for element in result.select(&h3_sel) {
            name_search_results.push(element.text().collect::<Vec<_>>().join(""));

            for link in element.select(&link_sel) {
                anime_ids.push(link.value().attr("href").unwrap()
                   .split('-').last().unwrap()
                   .split_once('?').unwrap().0);
            }
        }
    }


    if name_search_results.is_empty() {
        println!("{}No results found", "\x1b[31m");
        std::process::exit(0)
    }

    let name = rust_fzf::select(name_search_results.clone(), vec!["--reverse".to_string()]);

    if name.is_empty() {
        println!("{}Exiting...", "\x1b[33m");
        std::process::exit(0)
    } 

    select_episode(
        anime_ids[name_search_results.iter().position(|x| x == &name).unwrap()],
        name
    )
}


fn select_episode(anime_id: &str, name: String) -> Ani {
    let episodes_url = format!("https://aniwatch.to/ajax/v2/episode/list/{}", anime_id);
    let episodes_json: Value = serde_json::from_str(get_response(&episodes_url).unwrap().as_str()).unwrap();

    let episodes_html = Html::parse_document(episodes_json["html"].as_str().unwrap());
    let ep_sel = Selector::parse("a.ssl-item").unwrap();

    let mut all_episode_ids: Vec<u32> = Vec::new();
    for element in episodes_html.select(&ep_sel) {
        all_episode_ids.push(element.value()
            .attr("data-id").unwrap()
            .parse::<u32>().unwrap()
        )
    }

    let episode_num = rust_fzf::select(
        (1..=episodes_json["totalItems"].as_u64().unwrap()).map(|x| x.to_string()).collect(),
        vec!["--reverse".to_string()]
    ).parse::<usize>().unwrap_or_else(|_| {println!("{}Exiting...", "\x1b[33m"); std::process::exit(0)});


    Ani {
        ep_ids: Some(all_episode_ids),
        name: name,
        ep: episode_num,
        id: anime_id.parse().unwrap()
    }
}


pub fn update_ep_ids(anime_id: usize) -> Option<Vec<u32>> {
    let episodes_url = format!("https://aniwatch.to/ajax/v2/episode/list/{}", anime_id);
    let episodes_json: Value = serde_json::from_str(get_response(&episodes_url).unwrap().as_str()).unwrap();

    let episodes_html = Html::parse_document(episodes_json["html"].as_str().unwrap());
    let ep_sel = Selector::parse("a.ssl-item").unwrap();

    Some(episodes_html.select(&ep_sel).map(|x| x.value().attr("data-id").unwrap().parse::<u32>().unwrap()).collect())
}