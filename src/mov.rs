use scraper::{Html, Selector};
use mov_mod::Mov;
mod mov_mod;

pub fn search_movie() {
    let mut query = String::new();
    println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
    std::io::stdin().read_line(&mut query).expect("reading stdin");
    let mut mov = select_movie(&query.replace(" ", "-"));

    loop {
        mov.play();

        let select = rust_fzf::select(
            vec!["next".to_string(), "previous".to_string(), "search".to_string(), "quit".to_string()], 
            vec!["--reverse".to_string(), format!("--header=Current ep - {} of {}", mov.ep, mov.name)]).to_string();

        match select.as_str() {
            "next" => {mov.ep += 1; if mov.ep >= mov.ep_ids.len() { println!("{}Episode out of bound", "\x1b[31m") } std::process::exit(0)  },
            "previous" => {mov.ep -= 1; if mov.ep <= 0 { println!("{}Episode out of bound", "\x1b[31m") } std::process::exit(0) },
            "search" => {
                let mut query = String::new();
                println!("{}Search for movie/tv show: {}", "\x1b[34m", "\x1b[0m");
                std::io::stdin().read_line(&mut query).expect("reading stdin");
                mov = select_movie(&query.replace(" ", "-"));
            },
            "quit" => std::process::exit(0),
            _ => ()
        }
    }
}


fn select_movie(query: &str) -> Mov {
    let response = get_response(format!("https://flixhq.to/search/{}", query)).unwrap();

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
                a_elem.attr("href").unwrap().split("/").skip(1).next().unwrap()
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
        get_movie_server_id(movie_id, name)
    } else {
        select_episode(
            select_season(movie_id).as_str(),
            name
        )
    }
}

fn get_movie_server_id(movie_id: &str, name: String) -> Mov {
    let response = get_response(format!("https://flixhq.to/ajax/movie/episodes/{}", movie_id)).unwrap();
    let page = Html::parse_document(&response);

    let sel = Selector::parse("a").unwrap();

    let mut id = 0;
    for i in page.select(&sel) {
        id = i.value().attr("data-linkid").unwrap().parse::<u32>().unwrap();
        break
    }


    Mov {
        ep_ids: vec![id],
        name: name,
        ep: 1
    }
}

fn select_season(movie_id: &str) -> String {
    let response = get_response(format!("https://flixhq.to/ajax/v2/tv/seasons/{}", movie_id)).unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let seasons_page = Html::parse_document(&response);

    let mut seasons_all: Vec<String> = Vec::new();
    let mut season_ids: Vec<String> = Vec::new();


    for i in seasons_page.select(&a_sel) {
        seasons_all.push(i.text().collect::<Vec<_>>().join(""));
        season_ids.push(i.value().attr("data-id").unwrap().to_string())
    }
    let season = rust_fzf::select(seasons_all.clone(), vec!["--reverse".to_string()]);


    season_ids[seasons_all.iter().position(|s| s == &season).unwrap()].to_string()
}

fn select_episode(season_id: &str, name: String) -> Mov {
    let response = get_response(format!("https://flixhq.to/ajax/v2/season/episodes/{}", season_id)).unwrap();
    let episodes_page = Html::parse_document(&response);
    let a_sel = Selector::parse("a").unwrap();

    let mut all_episode_ids: Vec<u32> = Vec::new();
    for element in episodes_page.select(&a_sel) {
        all_episode_ids.push(element.value().attr("data-id").unwrap().parse::<u32>().unwrap())
    }

    let episode_num = rust_fzf::select(
        (1..=all_episode_ids.len()).map(|x| x.to_string()).collect(),
        vec!["--reverse".to_string()]
    ).parse::<usize>().unwrap();


    Mov {
        ep_ids: all_episode_ids,
        name: name,
        ep: episode_num
    }
}

#[tokio::main] 
async fn get_response(url: String) -> Result<String, reqwest::Error> {
    Ok(reqwest::get(url)
       .await?
       .text()
       .await?
    )
}