use std::path::PathBuf;

pub struct ServeConfig {
    pub theme: PathBuf,
}

pub struct Config {
    sources: Vec<PathBuf>,
    pub output: PathBuf,
    pub db_path: PathBuf,
    pub frontpage: Option<String>,
    pub theme: Option<PathBuf>,
    pub serve: ServeConfig,
}

impl Config {
    pub fn new() -> Config {
        Config {
            sources: vec![],
            output: PathBuf::from(&"out/"),
            db_path: PathBuf::from(&"db.sqlite3"),
            frontpage: None,
            theme: None,
            serve: ServeConfig {
                theme: PathBuf::from("theme/default"),
            },
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
