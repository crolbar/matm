use crate::utils::get_response;
use crate::ani::ani_mod::Ani;
use std::collections::HashMap;
use scraper::{Html, Selector};
use serde_json::Value;


pub fn select_anime(query: &str) -> std::io::Result<Ani> {
    let div_sel = Selector::parse("div.film_list-wrap").unwrap();
    let link_sel = Selector::parse("a.dynamic-name").unwrap();

    let mut anime_result: HashMap<String, String>= HashMap::new();

    let mut page_num = 1;
    while page_num < 5 { 
        let response = get_response(&format!("https://aniwatch.to/search?keyword={}&page={}", query, page_num)).unwrap_or_else(|_| { println!("{}No internet connection", "\x1b[33m"); std::process::exit(0) });

        let search_page = Html::parse_document(response.as_str());
        let search_results = search_page.select(&div_sel).next().unwrap();

        let elem_iter = search_results.clone().select(&link_sel);
        if elem_iter.clone().count() == 0 { break }

        for element in elem_iter {
            anime_result.insert(
                element.value().attr("href").unwrap()
                .split('-').last().unwrap()
                .split_once('?').unwrap().0.to_string(),

                element.text().collect::<Vec<_>>().join("")
            );
        }

        page_num += 1;
    }

    if anime_result.is_empty() {
        println!("{}No results found", "\x1b[31m");
        std::process::exit(0)
    }

    let name = match selector::select(
        anime_result.iter().map(|x| x.1.to_string()).collect(),
        None, None
    )? {
        name if name.is_empty() => {
            println!("{}Exiting...", "\x1b[33m");
            std::process::exit(0);
        }
        name => name
    };

    Ok(
        select_episode(
            anime_result.iter().find(|i| i.1 == &name).unwrap().0.to_string(),
            name
        )?
    )
}


fn select_episode(anime_id: String, name: String) -> std::io::Result<Ani> {
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

    let episode_num = selector::select(
        (1..=episodes_json["totalItems"].as_u64().unwrap()).map(|x| x.to_string()).collect(),
        None, None
    )?.parse::<usize>().unwrap_or_else(|_| {
        println!("{}Exiting...", "\x1b[33m");
        std::process::exit(0)
    });


    Ok(
        Ani {
            ep_ids: Some(all_episode_ids),
            name,
            ep: episode_num,
            id: anime_id.parse().unwrap()
        }
    )
}

pub fn update_ep_ids(anime_id: usize) -> Option<Vec<u32>> {
    let episodes_url = format!("https://aniwatch.to/ajax/v2/episode/list/{}", anime_id);
    let episodes_json: Value = serde_json::from_str(get_response(&episodes_url).unwrap_or_else(|_| { println!("{}No internet connection", "\x1b[33m"); std::process::exit(0) }).as_str()).unwrap();

    let episodes_html = Html::parse_document(episodes_json["html"].as_str().unwrap());
    let ep_sel = Selector::parse("a.ssl-item").unwrap();

    Some(episodes_html.select(&ep_sel).map(|x| x.value().attr("data-id").unwrap().parse::<u32>().unwrap()).collect())
}
