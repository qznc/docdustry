use std::path::PathBuf;

pub struct Config {
    sources: Vec<PathBuf>,
    pub output: PathBuf,
    pub frontpage: Option<String>,
    pub theme: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            sources: vec![],
            output: PathBuf::from(&"out/"),
            frontpage: None,
            theme: None,
        }
    }

    pub fn get_sources(&self) -> Vec<PathBuf> {
        if self.sources.is_empty() {
            vec![PathBuf::from(&".")]
        } else {
            self.sources.clone()
        }
    }

    pub fn push_source_dir(&mut self, s: PathBuf) {
        self.sources.push(s)
    }
}
