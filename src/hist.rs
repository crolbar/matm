use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufReader;
use crate::ani::Ani;
use crate::mov::Mov;
use crate::man::Man;
use std::fs::File;

pub enum DataType {
    AniData,
    MovData,
    ManData
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hist {
    pub ani_data: Vec<Ani>,
    pub mov_data: Vec<Mov>,
    pub man_data: Vec<Man>

}

impl Hist {
    pub fn serialize(&self) {
        ron::ser::to_writer_pretty(Hist::get_file(true), self, ron::ser::PrettyConfig::new())
            .unwrap()
    } 
    pub fn deserialize() -> Self {
        ron::de::from_reader(BufReader::new(Hist::get_file(false)))
            .unwrap_or_else(|_| {
                println!("{}The history file is empty or has wrong ron syntax, try to delete it and try again.", "\x1b[31m");
                std::process::exit(1) 
            })
    }

    pub fn ani_save(ani: Ani) {
        let mut hist = Hist::deserialize();

        let ani = Ani { 
            ep_ids: None,
            ep: ani.ep + 1,
            sel_provider: String::new(),
            providers: HashMap::new(),
            ..ani
        };

        match hist.ani_data.iter().position(|x| x.name == ani.name) {
            Some(pos) => hist.ani_data[pos].ep = ani.ep,
            None => hist.ani_data.push(ani) 
        }

        hist.serialize()
    }

    pub fn man_save(man: Man) {
        let mut hist = Hist::deserialize();

        let man = Man {
            all_chapters: vec![],
            chapter: man.all_chapters[man.all_chapters.iter().position(|x| x == &man.chapter).unwrap() + 1].clone(),
            ..man 
        };
        match hist.man_data.iter().position(|x| x.name == man.name) {
            Some(pos) => hist.man_data[pos].chapter = man.chapter,
            None => hist.man_data.push(man) 
        }

        hist.serialize()
    }

    pub fn mov_save(mov: Mov) {
        let mut hist = Hist::deserialize();

        let mov = Mov {
            ep_ids: None,
            ep: mov.ep + 1,
            providers: HashMap::new(),
            sel_provider: String::new(),
            ..mov 
        };
        match hist.mov_data.iter().position(|x| x.name == mov.name) {
            Some(pos) => hist.mov_data[pos].ep = mov.ep,
            None => hist.mov_data.push(mov) 
        }

        hist.serialize()
    }

    pub fn remove(name: &str, dt: DataType) {
        let mut hist = Hist::deserialize();
        match dt {
            DataType::AniData => { hist.ani_data.remove(hist.ani_data.iter().position(|x| x.name == name).unwrap()); },
            DataType::MovData => { hist.mov_data.remove(hist.mov_data.iter().position(|x| x.name == name).unwrap()); },
            DataType::ManData => { hist.man_data.remove(hist.man_data.iter().position(|x| x.name == name).unwrap()); },
        }
        hist.serialize()
    }

    fn get_file(is_ser: bool) -> File {
        let dir_path = dirs::home_dir().unwrap().join(".local/state/matm");
        let file_path = dir_path.join("hist.ron");

        if File::open(&file_path).is_err() {
            if !dir_path.exists() { std::fs::create_dir_all(&dir_path).unwrap() }
            File::create(&file_path).unwrap();
            Hist { ani_data: vec![], mov_data: vec![], man_data: vec![] }.serialize()
        }

        match is_ser {
            true => File::create(&file_path).unwrap(),
            false => File::open(&file_path).unwrap()
        }
    }

    pub fn delete_hist(dt: DataType) {
        let mut hist = Hist::deserialize();
        match dt {
            DataType::AniData => hist.ani_data.clear(),
            DataType::MovData => hist.mov_data.clear(),
            DataType::ManData => hist.man_data.clear(),
        }
        hist.serialize();
        println!("{}History deleted", "\x1b[34m")
    }
}
