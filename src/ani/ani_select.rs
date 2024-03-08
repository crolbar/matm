use crate::utils::get_response;
use scraper::{Html, Selector};
use serde_json::Value;
use crate::ani::Ani;

impl Ani {
    pub fn select_anime(query: &str) -> std::io::Result<Ani> {
        let div_sel = Selector::parse("div.film_list-wrap").unwrap();
        let item_sel = Selector::parse("div.flw-item").unwrap();
        let num_eps_sel = Selector::parse("div.tick-item").unwrap();
        let link_sel = Selector::parse("a.dynamic-name").unwrap();

        let mut names: Vec<String> = Vec::new();
        let mut ids: Vec<String> = Vec::new();

        for page_num in 1..=5 { 
            let url = format!("https://aniwatchtv.to/search?keyword={}&page={}", query, page_num);
            let response = get_response(&url)
                .unwrap_or_else(|_| {
                    println!("{}No internet connection", "\x1b[33m");
                    std::process::exit(0) 
                });

            let search_page = Html::parse_document(response.as_str());
            let search_results = search_page.select(&div_sel).next().unwrap();

            let elem_iter = search_results.clone().select(&item_sel);
            if elem_iter.clone().count() == 0 { break }

            for element in elem_iter {
                let link = element.select(&link_sel).next().unwrap();
                let mut name = link.text().collect::<Vec<_>>().join("");

                if  names.contains(&name) {
                    let num_of_eps_div = element.select(&num_eps_sel).next().unwrap();
                    let num_of_eps = num_of_eps_div.text().collect::<String>();
                    name.push_str(format!(" (EPS: {})", num_of_eps).as_str());
                }

                names.push(name);
                ids.push(
                    link.value().attr("href").unwrap()
                    .split('-').last().unwrap()
                    .split_once('?').unwrap().0.to_string()
                );
            }
        }

        if names.is_empty() {
            println!("{}No results found", "\x1b[31m");
            std::process::exit(0)
        }

        let name = 
            selector::select(
                names.iter().map(|x| x.to_string()).collect(),
                None, None
            )?;

        if name.is_empty() {
            println!("{}Exiting...", "\x1b[33m");
            std::process::exit(0);
        }

        Ok(
            Self::select_episode(
                ids[names.iter().position(|i| *i == name).unwrap()]
                    .parse().unwrap(),
                name
            )?
          )
    }


    fn select_episode(id: usize, name: String) -> std::io::Result<Ani> {
        let episodes_url = format!("https://aniwatchtv.to/ajax/v2/episode/list/{}", id);
        let episodes_json: Value = serde_json::from_str(&get_response(&episodes_url).unwrap()).unwrap();

        let episodes_html = Html::parse_document(episodes_json["html"].as_str().unwrap());
        let ep_sel = Selector::parse("a.ssl-item").unwrap();

        let ep_ids: Vec<u32> = 
            episodes_html.select(&ep_sel).map(|el| {
                el.value()
                    .attr("data-id").unwrap()
                    .parse::<u32>().unwrap()
            }).collect();

        let ep = selector::select(
            (1..=episodes_json["totalItems"].as_u64().unwrap()).map(|x| x.to_string()).collect(),
            None, None
            )?.parse::<usize>().unwrap_or_else(|_| {
            println!("{}Exiting...", "\x1b[33m");
            std::process::exit(0)
        });

        Ok(Ani { 
            ep_ids, name, ep, id,
            ..Default::default()
        })
    }
}
