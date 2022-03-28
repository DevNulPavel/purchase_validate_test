use helpers_lib::{deserialize_string_not_empty, deserialize_url};
use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProjectInfo {
    #[serde(deserialize_with = "deserialize_url")]
    pub api_url: Url,

    #[serde(deserialize_with = "deserialize_string_not_empty")]
    pub secret_key: String,

    #[serde(deserialize_with = "deserialize_string_not_empty")]
    pub name: String,
}