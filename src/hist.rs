use serde::{Deserialize, Serialize};
use crate::ani::ani_mod::Ani;
use crate::mov::mov_mod::Mov;
use std::io::BufReader;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hist {
    pub ani_data: Vec<Ani>,
    pub mov_data: Vec<Mov>

}

impl Hist {
    pub fn serialize(&self) {
        ron::ser::to_writer_pretty(Hist::get_file(true), self, ron::ser::PrettyConfig::new())
            .unwrap()
    } 
    pub fn deserialize() -> Self {
        ron::de::from_reader(BufReader::new(Hist::get_file(false)))
            .unwrap_or_else(|_| { println!("{}The history file is empty or has wrong ron syntax, try to delete it and try again.", "\x1b[31m"); std::process::exit(1) } )
    }

    pub fn ani_save(ani: Ani) {
        let mut hist = Hist::deserialize();

        let ani = Ani { ep_ids: None, ep: ani.ep + 1, ..ani };
        match hist.ani_data.iter().position(|x| x.name == ani.name) {
            Some(pos) => hist.ani_data[pos].ep = ani.ep,
            None => hist.ani_data.push(ani) 
        }

        hist.serialize()
    }

    pub fn mov_save(mov: Mov) {
        let mut hist = Hist::deserialize();

        let mov = Mov { ep_ids: None, ep: mov.ep + 1, ..mov };
        match hist.mov_data.iter().position(|x| x.name == mov.name) {
            Some(pos) => hist.mov_data[pos].ep = mov.ep,
            None => hist.mov_data.push(mov) 
        }

        hist.serialize()
    }

    pub fn remove(name: &str, is_ani: bool) {
        let mut hist = Hist::deserialize();
        match is_ani {
            true => { hist.ani_data.remove(hist.ani_data.iter().position(|x| x.name == name).unwrap()); },
            false => { hist.mov_data.remove(hist.mov_data.iter().position(|x| x.name == name).unwrap()); },
        }
        hist.serialize()
    }

    fn get_file(is_ser: bool) -> File {
        let dir_path = dirs::home_dir().unwrap().join(".local/state/mani");
        let file_path = dir_path.join("hist.ron");

        if File::open(&file_path).is_err() {
            if !dir_path.exists() { std::fs::create_dir_all(&dir_path).unwrap() }
            File::create(&file_path).unwrap();
            Hist { ani_data: vec![], mov_data: vec![] }.serialize()
        }

        match is_ser {
            true => File::create(&file_path).unwrap(),
            false => File::open(&file_path).unwrap()
        }
    }
}