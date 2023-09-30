use std::process::Command;
use scraper::{Html, Selector};
use serde_json::Value;



fn main() {
    search();
    
}

#[derive(Debug)]
struct Ani {
    ep_ids: Vec<u32>,
    name: String,
    ep: usize
}


impl Ani {
    fn play(&mut self) {
        let sources = get_sources(&self);
        Command::new("mpv")
            .args([
                sources.0,
                format!("--sub-file={}",sources.1),
                format!("--force-media-title={} Episode {}", self.name, self.ep
            )])
            .spawn().expect("crashed when trying to start mpv")
            .wait().unwrap();


        self.next_prev()
    }


    fn next_prev(&mut self) {
        let sel = vec!["next".to_string(), "previous".to_string(), "search".to_string(), "quit".to_string()];

        let sel = rust_fzf::select(sel, vec!["--reverse".to_string()]).to_string();

        match sel.as_str() {
            "next" => {self.ep += 1; self.play()},
            "previous" => {self.ep -= 1; self.play()},
            "search" => search(),
            "quit" => std::process::exit(0),
            _ => ()
        }
    }
}



fn search() {
    // let mut s = String::new();
    // std::io::stdin().read_line(&mut s).expect("reading stdin");
    let s = "rent a girlfriend";

    let url = format!("https://aniwatch.to/search?keyword={}", s);
    let response = get_response(url).unwrap();


    let mut ani = select_anime(&response);

    ani.play();
}


fn get_sources(ani: &Ani) -> (String, String) {
    let url = format!("https://aniwatch.to/ajax/v2/episode/sources?id={}", get_ep_data_id(ani.ep_ids[ani.ep - 1]));
    let provider: Value = serde_json::from_str(get_response(url).expect("crashed at trying to get provider").as_str()).unwrap();
    let provider_url = url::Url::parse(provider["link"].as_str().unwrap()).unwrap();

    let url = format!("https://{}/embed-2/ajax/e-1/getSources?id={}",
        provider_url.host_str().unwrap(),
        provider_url.path().rsplit('/').next().unwrap_or("")
    );
    let sources_json: Value = serde_json::from_str(get_response(url).unwrap().as_str()).unwrap();


    let enc_video_url: String = sources_json["sources"].to_string().chars().filter(|x| x.to_string() != "\"").collect();
    let encrypted = sources_json["encrypted"].as_bool().unwrap();

    let mut sub_source = String::new();
    if let Some(s) = sources_json["tracks"].as_array().unwrap().iter()
        .find(|v| v["label"] == "English") {
            sub_source = s["file"].to_string()
                .chars().filter(|x| x.to_string() != "\"")
                .collect();
        };


    // println!("{}", enc_video_url.as_str());

    if !encrypted {
        let source: Value = serde_json::from_str(enc_video_url.as_str()).unwrap();
        let source = source["file"].to_string().chars().filter(|x| x.to_string() != "\"").collect();
        return (source, sub_source);
    }



    (decrypt_url(enc_video_url), sub_source)
}

fn select_anime(response: &String) -> Ani {
    let div_sel = Selector::parse("div.film_list-wrap").unwrap();
    let h3_sel = Selector::parse("h3.film-name").unwrap();
    let link_sel = Selector::parse("a.dynamic-name").unwrap();


    let search_page = Html::parse_document(response);
    let search_results = search_page.select(&div_sel);

    let mut names:Vec<String> = Vec::new();
    let mut ids:Vec<&str> = Vec::new();



    for result in search_results {
        for element in result.select(&h3_sel) {
            names.push(element.text().collect::<Vec<_>>().join(""));

            for link in element.select(&link_sel) {
                ids.push(link.value().attr("href").unwrap()
                   .split('-').last().unwrap()
                   .split_once('?').unwrap().0);
            }
        }
    }

    let name = rust_fzf::select(names.clone(), vec!["--reverse".to_string()]);
    

    let mut id = "";
    for i in names.iter().enumerate() {
        if names[i.0] == name {
            id = ids[i.0];
        }
    }


    let episodes_url = format!("https://aniwatch.to/ajax/v2/episode/list/{}", id);
    let episodes_json: Value = serde_json::from_str(get_response(episodes_url).unwrap().as_str()).unwrap();


    let episode_num = rust_fzf::select(
        (1..=episodes_json["totalItems"].as_u64().unwrap()).map(|x| x.to_string()).collect(),
            vec!["--reverse".to_string()]
    ).parse::<usize>().unwrap();


    let episodes_html = Html::parse_document(episodes_json["html"].as_str().unwrap());
    let ep_sel = Selector::parse("a.ssl-item").unwrap();


    let mut episode_ids: Vec<u32> = Vec::new();
    for element in episodes_html.select(&ep_sel) {
        episode_ids.push(element.value().attr("data-id").unwrap().parse::<u32>().unwrap())
    }


    Ani {
        ep_ids: episode_ids,
        name: name,
        ep: episode_num
    }
}

fn get_ep_data_id(ep_id: u32) -> u32 {
    let url = format!("https://aniwatch.to/ajax/v2/episode/servers?episodeId={}", ep_id);
    let data_ids_req = get_response(url).unwrap();
    data_ids_req.split_once("data-id=\\\"").unwrap().1
        .split_once("\\\"\\n").unwrap().0
        .parse::<u32>().unwrap()
}

fn decrypt_url(url: String) -> String {
    let key: Vec<Vec<u32>> = serde_json::from_str(&get_response("https://raw.githubusercontent.com/enimax-anime/key/e6/key.txt".to_string()).expect("couldnt get key")).expect("couldnt deserialize string to vec");
    let mut extracted_key = String::new();
    let mut enc_url: Vec<char> = url.chars().collect();
    
    for i in key {
        for j in i[0]..i[1] {
            extracted_key.push(enc_url[j as usize]);
            enc_url[j as usize] = ' '
        }
    }
    let enc_url_without_key: String = enc_url.iter().filter(|&&c| !c.is_whitespace()).collect();




    let cmd = format!(
        "echo {} | base64 -d | openssl enc -aes-256-cbc -d -md md5 -k {} 2>/dev/null | sed -nE 's_.*\"file\":\"([^\"]*)\".*_\\1_p'",
        enc_url_without_key, extracted_key
    );

    // Use the Command module to execute the shell command
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute command");





    String::from_utf8(output.stdout).expect("Failed to convert to string")
}

#[tokio::main] 
async fn get_response(url: String) -> Result<String, reqwest::Error> {
    Ok(reqwest::get(url)
       .await?
       .text()
       .await?
    )
}