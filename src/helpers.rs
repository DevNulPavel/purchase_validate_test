use reqwest::Url;
use serde::{de::Error, Deserialize, Deserializer};
use std::borrow::Cow;

pub fn deserialize_url<'de, D>(data: D) -> Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let text = Cow::<str>::deserialize(data)?;
    //let text: Cow<str> = Deserialize::deserialize(data)?;

    Url::parse(&text).map_err(Error::custom)
}

pub fn deserialize_string_not_empty<'de, D>(data: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(data)?;
    if text.is_empty() || text.eq("~") || text.eq("null") {
        return Err(Error::invalid_length(0, &"length > 0"));
    }

    Ok(text)
}