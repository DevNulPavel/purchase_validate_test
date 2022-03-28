use super::{project_info::ProjectInfo, TestCase};
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::Path};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub project: ProjectInfo,
    pub tests: Vec<TestCase>,
}

impl Config {
    /// Пытаемся распасить конфиг из файлика
    pub fn parse_from_file(path: &Path) -> Result<Config, eyre::Error> {
        // Пробуем загрузить конфиг из файлика в зависимости от расширения
        let config: Config = match path
            .extension()
            .and_then(|v| v.to_str())
            .map(str::to_lowercase)
            .as_deref()
        {
            Some("yml") | Some("yaml") => {
                let r = BufReader::new(File::open(path)?);
                serde_yaml::from_reader(r)?
            }
            Some("json") => {
                let r = BufReader::new(File::open(path)?);
                serde_json::from_reader(r)?
            }
            _ => {
                return Err(eyre::eyre!(
                    "Unsupported config file extention {}. Only yml/yaml/json/toml are supported",
                    path.display()
                ));
            }
        };
        Ok(config)
    }
}
