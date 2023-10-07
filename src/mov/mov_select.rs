use crate::utils::get_response;
use crate::mov::mov_mod::Mov;
use scraper::{Html, Selector};


pub fn select_movie_show(query: &str) -> Mov {
    let response = get_response(&format!("https://flixhq.to/search/{}", query)).unwrap();

    let div_sel = Selector::parse("div.film_list-wrap").unwrap();
    let detail_sel = Selector::parse("div.film-detail").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let info_sel = Selector::parse("span.fdi-item").unwrap();


    let search_page = Html::parse_document(&response);
    let search_results = search_page.select(&div_sel);

    let mut name_search_results:Vec<String> = Vec::new();
    let mut movie_ids:Vec<&str> = Vec::new();

    for result in search_results {
        for element in result.select(&detail_sel) {
            let a_elem = element.select(&a_sel).next().unwrap().value();
            let last_info_elem = element.select(&info_sel).last().unwrap().text().collect::<Vec<_>>().join("");
            name_search_results.push(format!("{} ({}) ({})",
                a_elem.attr("title").unwrap().to_string(),
                if last_info_elem.contains("EPS") { last_info_elem } else { element.select(&info_sel).next().unwrap().text().collect::<Vec<_>>().join("") },
                a_elem.attr("href").unwrap().split("/").skip(1).next().unwrap() // CHANGE THIS !! how we diff movie from tv show
            ));

            movie_ids.push(a_elem.attr("href").unwrap().rsplit_once('-').unwrap().1)
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


    let movie_id = movie_ids[name_search_results.iter().position(|x| x == &name).unwrap()];

    if name.contains("movie") {
        get_movie_server_id(
            movie_id,
            name
        )
    } else {
        let season = select_season(movie_id, name);
        select_episode(
            season.0,
            season.1
        )
    }
}

fn get_movie_server_id(movie_id: &str, name: String) -> Mov {
    let response = get_response(&format!("https://flixhq.to/ajax/movie/episodes/{}", movie_id)).unwrap();
    let page = Html::parse_document(&response);

    let sel = Selector::parse("a").unwrap();


    let server_ids: Vec<String> = page.select(&sel).map(|x| x.value().attr("data-linkid").unwrap().to_string()).collect();


    Mov {
        ep_ids: Some(server_ids),
        season_id: None,
        name: name,
        ep: 1
    }
}

fn select_season(movie_id: &str, name: String) -> (String, String) {
    let response = get_response(&format!("https://flixhq.to/ajax/v2/tv/seasons/{}", movie_id)).unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let seasons_page = Html::parse_document(&response);

    let mut seasons_all: Vec<String> = Vec::new();
    let mut season_ids: Vec<String> = Vec::new();


    for i in seasons_page.select(&a_sel) {
        seasons_all.push(i.text().collect::<Vec<_>>().join(""));
        season_ids.push(i.value().attr("data-id").unwrap().to_string())
    }
    let season = rust_fzf::select(seasons_all.clone(), vec!["--reverse".to_string()]);


    (season_ids[seasons_all.iter().position(|s| s == &season).unwrap()].to_string(), format!("{} {}", name, season))
}

fn select_episode(season_id: String, name: String) -> Mov {
    let response = get_response(&format!("https://flixhq.to/ajax/v2/season/episodes/{}", season_id)).unwrap();
    let episodes_page = Html::parse_document(&response);
    let a_sel = Selector::parse("a").unwrap();

    let all_episode_ids: Vec<String> = episodes_page.select(&a_sel).map(|x| x.value().attr("data-id").unwrap().to_string()).collect();

    let episode_num = rust_fzf::select(
        (1..=all_episode_ids.len()).map(|x| x.to_string()).collect(),
        vec!["--reverse".to_string()]
    ).parse::<usize>().unwrap();


    Mov {
        ep_ids: Some(all_episode_ids),
        season_id: Some(season_id.parse().unwrap()),
        name: name,
        ep: episode_num
    }
}

pub fn update_ep_ids(season_id: usize) -> Option<Vec<String>> {
    let response = get_response(&format!("https://flixhq.to/ajax/v2/season/episodes/{}", season_id)).unwrap();
    let episodes_page = Html::parse_document(&response);
    let a_sel = Selector::parse("a").unwrap();

    Some(episodes_page.select(&a_sel).map(|x| x.value().attr("data-id").unwrap().to_string()).collect())
}