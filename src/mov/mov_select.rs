use crate::utils::get_response;
use std::collections::HashMap;
use scraper::{Html, Selector};
use crate::mov::Mov;


impl Mov {
    pub fn select_movie_show(query: &str) -> std::io::Result<Self> {
        let response = get_response(&format!("https://flixhq.to/search/{}", query))
            .unwrap_or_else(|_| {
                println!("{}No internet connection", "\x1b[33m");
                std::process::exit(0) 
            });

        let div_sel = Selector::parse("div.film_list-wrap").unwrap();
        let detail_sel = Selector::parse("div.film-detail").unwrap();
        let a_sel = Selector::parse("a").unwrap();
        let info_sel = Selector::parse("span.fdi-item").unwrap();


        let search_page = Html::parse_document(&response);
        let search_results = search_page.select(&div_sel);

        let mut movie_search_results: HashMap<String, &str> = HashMap::new();
        for result in search_results {
            for element in result.select(&detail_sel) {
                let a_elem = element.select(&a_sel).next().unwrap().value();

                let last_info_elem = element.select(&info_sel).last().unwrap().text().collect::<Vec<_>>().join("");

                movie_search_results.insert(
                    format!("{} ({}) ({})",
                        a_elem.attr("title").unwrap(),

                        match !last_info_elem.contains("EPS") {
                            true => element.select(&info_sel).next().unwrap().text().collect::<Vec<_>>().join(""),
                            false => last_info_elem 
                        },

                        match a_elem.attr("href").unwrap().contains("/movie/") {
                            true => "movie",
                            false => match a_elem.attr("href").unwrap().contains("/tv/") {
                                    true => "tv",
                                    false => "unkown"
                                }
                        }
                    ),

                    a_elem.attr("href").unwrap().rsplit_once('-').unwrap().1
                );
            }
        }

        if movie_search_results.is_empty() {
            println!("{}No results found", "\x1b[31m");
            std::process::exit(0)
        }

        let name = selector::select(
            movie_search_results.iter().map(|(name, _)| name.to_string()).collect(),
            None, None
        ).unwrap();

        if name.is_empty() {
            println!("{}Exiting...", "\x1b[33m");
            std::process::exit(0)
        } 

        let movie_id = movie_search_results.get(&name).unwrap();

        Ok(
            if name.contains("movie") {
                Mov::get_movie_server_id(
                    movie_id,
                    name
                )
            } else {
                let season = Mov::select_season(movie_id, &name);

                Mov::select_episode(
                    Some(season.0.parse().unwrap()), // season id 
                    season.1 // name
                )
            }
          )
    }

    fn select_season<'s>(movie_id: &'s str, name: &'s str) -> (String, String) {
        let response = get_response(&format!("https://flixhq.to/ajax/v2/tv/seasons/{}", movie_id)).unwrap();
        let a_sel = Selector::parse("a").unwrap();
        let seasons_page = Html::parse_document(&response);

        let mut seasons: Vec<(String, &str)> = Vec::new();

        seasons_page.select(&a_sel).for_each(|i| {
            seasons.push(
                (
                    i.text().collect::<Vec<_>>().join(""),
                    i.value().attr("data-id").unwrap()
                )
            )
        });

        let season_num = 
            selector::select(
                seasons.iter().map(|i| i.0.to_string()).collect(),
                None, None
            ).unwrap();


        (
            seasons.iter().find(|i| i.0 == season_num).unwrap().1.to_string(), // season id
            format!("{} {}", name, season_num) // name
        )
    }

    fn select_episode(season_id: Option<usize>, name: String) -> Self {
        let url = format!("https://flixhq.to/ajax/v2/season/episodes/{}", season_id.unwrap());
        let response = get_response(&url).unwrap();
        let episodes_page = Html::parse_document(&response);
        let a_sel = Selector::parse("a").unwrap();

        let (ep_ids_len, ep_ids)= {
            let ep_ids = 
                episodes_page.select(&a_sel)
                .map(|x| x.value().attr("data-id").unwrap().to_string())
                .collect::<Vec<String>>();

            ( ep_ids.len(), Some(ep_ids))
        };

        let ep = 
            selector::select(
                (1..=ep_ids_len).map(|x| x.to_string()).collect(),
                None, None
            ).unwrap()
            .parse::<usize>().unwrap_or_else(|_|{
                println!("{}Exiting...", "\x1b[33m");
                std::process::exit(0)
            });

        Self { ep_ids, season_id, ep, name, ..Default::default() }
    }

    fn get_movie_server_id(movie_id: &str, name: String) -> Self {
        let response = get_response(&format!("https://flixhq.to/ajax/movie/episodes/{}", movie_id)).unwrap();
        let page = Html::parse_document(&response);

        let sel = Selector::parse("a").unwrap();

        let ep_ids: Option<Vec<String>> = Some(
            page.select(&sel)
                .map(|x| x.value().attr("data-linkid").unwrap().to_string())
                .collect());


        Self { ep_ids, season_id: None, ep: 1, name, ..Default::default()}
    }
}
