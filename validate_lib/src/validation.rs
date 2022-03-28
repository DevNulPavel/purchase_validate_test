use crate::test_case::TestCase;
use eyre::WrapErr;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use sha1::{digest::Digest, Sha1};
use slog::{debug, Logger};

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// Тело запроса к серверу
#[derive(Debug, Serialize)]
struct JsonRequestBody<'a> {
    project_name: &'a str,
    payment_info: String,
    payment_info_signature: String,
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)] // TODO: ???
#[derive(Debug, Deserialize)]
struct PurchaseStatus {
    status: String,
    description: Option<String>,
    payload: Option<Vec<String>>,
}

// #[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PurchaseResponseData {
    validation_result: String,
    validation_result_signature: String,
}

#[allow(dead_code)] // TODO: ???
#[derive(Debug, Deserialize)]
struct JsonResponse {
    message: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    timestamp: chrono::DateTime<chrono::Utc>,
    datetime: String,
    data: PurchaseResponseData,
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn calc_signature(data: &[u8], key: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.update(key);
    let hash_number = hasher.finalize();
    format!("{:x}", hash_number)
}

// Запускаем проверку покупки
pub async fn check_purchase(
    logger: &Logger,
    test: TestCase,
    http_client: &Client,
    project_name: &str,
    secret_key: &str,
    api_url: &Url,
) -> Result<(), eyre::Error> {
    // Данные о платеже и подпись
    let purchase_base64_string = {
        let purchase_json_string =
            serde_json::to_string(&test.purchase).wrap_err("Purchase info serialize failed")?;
        debug!(logger, "Request data: {purchase_json_string}");

        base64::encode(purchase_json_string)
    };
    let purchase_signature =
        calc_signature(purchase_base64_string.as_bytes(), secret_key.as_bytes());

    // TODO: Запрос должен был быть GET
    // Выполняем запрос
    let response_obj = http_client
        .post(api_url.clone())
        .json(&JsonRequestBody {
            project_name,
            payment_info: purchase_base64_string,
            payment_info_signature: purchase_signature,
        })
        .send()
        .await
        .wrap_err("Test request perform error")?;

    // Ответ от сервера
    let status = response_obj.status();
    let response_text_result = response_obj.text().await;

    // В зависимости от статуса идем дальше или выводим ошибку
    let response_text = if status.is_success() {
        response_text_result.wrap_err("Response body receive failed")?
    } else {
        let err = match response_text_result.ok() {
            Some(response_error_text) => {
                eyre::eyre!(
                    "Server response with status {} and text: {}",
                    status,
                    response_error_text
                )
            }
            None => {
                eyre::eyre!("Server response with status {}", status)
            }
        };
        return Err(err);
    };
    debug!(logger, "Received from server: {response_text}");

    // Парсим
    let response_data =
        serde_json::from_str::<JsonResponse>(&response_text).wrap_err("Json parsing failed")?;

    // Вычисляем подпись от данных ответа
    let calculated_signature = calc_signature(
        response_data.data.validation_result.as_bytes(),
        secret_key.as_bytes(),
    );

    // Проверяем подпись
    eyre::ensure!(
        calculated_signature == response_data.data.validation_result_signature,
        "Response signature invalid: calculated {} != received {}",
        calculated_signature,
        response_data.data.validation_result_signature
    );

    // Парсим
    let response_data = {
        let response_json_data = base64::decode(response_data.data.validation_result)
            .wrap_err("Response base64 decode failed")?;

        let response_json_string =
            std::str::from_utf8(&response_json_data).wrap_err("UTF-8 parsing failed")?;
        debug!(logger, "Received json text: {response_json_string}");

        serde_json::from_str::<PurchaseStatus>(response_json_string)
            .wrap_err("Response json parsing failed")?
    };

    eyre::ensure!(
        response_data.status == test.response.status,
        "Status invalid: received {} != required {}",
        response_data.status,
        test.response.status
    );

    Ok(())
}
