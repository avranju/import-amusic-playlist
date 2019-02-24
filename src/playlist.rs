use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::errors::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    title: String,
    artist: String,
    album: String,
    composer: String,
    language: String,
    year: u16,
    keywords: Vec<String>,
}

impl Entry {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn artist(&self) -> &str {
        &self.artist
    }

    pub fn album(&self) -> &str {
        &self.album
    }

    pub fn composer(&self) -> &str {
        &self.composer
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn year(&self) -> &u16 {
        &self.year
    }

    pub fn keywords(&self) -> &Vec<String> {
        &self.keywords
    }
}

pub fn read_playlist<P: AsRef<Path>>(path: P) -> Result<Vec<Entry>> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}
