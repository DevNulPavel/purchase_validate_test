use helpers_lib::{deserialize_string_not_empty, deserialize_url};
use reqwest::Url;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::PathBuf};
use validate_lib::TestCase;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_url")]
    pub api_url: Url,
    #[serde(deserialize_with = "deserialize_string_not_empty")]
    pub secret_key: String,
    #[serde(deserialize_with = "deserialize_string_not_empty")]
    pub project_name: String,
    pub tests: Vec<TestCase>,
}

impl Config {
    /// Пытаемся распасить конфиг из файлика
    pub fn parse_from_file(path: PathBuf) -> Result<Config, eyre::Error> {
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
