use helpers_lib::deserialize_string_not_empty;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct PurchaseData {
    #[serde(deserialize_with = "deserialize_string_not_empty")]
    pub platform: String,
    #[serde(deserialize_with = "deserialize_string_not_empty")]
    pub product_id: String,
    #[serde(deserialize_with = "deserialize_string_not_empty")]
    pub order_id: String,
    #[serde(deserialize_with = "deserialize_string_not_empty")]
    pub receipt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_identifier: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ResponseData {
    #[serde(deserialize_with = "deserialize_string_not_empty")]
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct TestCase {
    pub purchase: PurchaseData,
    pub response: ResponseData,
}
