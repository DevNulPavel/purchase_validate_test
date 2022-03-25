mod app_arguments;
mod app_config;
mod helpers;

use crate::{
    app_arguments::AppArguments,
    app_config::{Config, TestCase},
};
use eyre::WrapErr;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use sha1::{digest::Digest, Sha1};
use slog::{crit, debug, info, trace, Drain, Level, Logger};
use slog_async::OverflowStrategy;
// use std::sync::{Arc};

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn init_logs(app_arguments: &AppArguments) -> Logger {
    /*const LOG_VAR: &str = "RUST_LOG";
    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "info");
    }
    pretty_env_logger::try_init().expect("Logger setup failed");*/

    let level = match app_arguments.verbose {
        0 => Level::Info,
        1 => Level::Debug,
        2 => Level::Trace,
        _ => panic!("Invalid verbose level"),
    };

    let term_decor = slog_term::TermDecorator::new().stdout().build();
    let term_drain = slog_term::FullFormat::new(term_decor)
        .use_file_location()
        .build()
        .fuse();

    // let json_drain = std::sync::Mutex::new(
    //     slog_json::Json::new(std::io::stdout())
    //         .add_default_keys()
    //         .build()
    //         .fuse(),
    // );

    // let dublicate_drain = slog::Duplicate(term_drain, json_drain).fuse();

    let drain = slog_async::Async::new(term_drain)
        .overflow_strategy(OverflowStrategy::Block)
        .build()
        .filter_level(level)
        .fuse();

    slog::Logger::root(drain, slog::o!())

    // let _log_guard = slog_scope::set_global_logger(logger);
    // let _guard = slog_stdlog::init().expect("Slog as log backend");
}

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

// Запускаем проверку покупки
async fn check_purchase(
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
    let purchase_signature = {
        let mut hasher = Sha1::new();
        hasher.update(purchase_base64_string.as_bytes());
        hasher.update(secret_key.as_bytes());
        let hash_number = hasher.finalize();
        format!("{:x}", hash_number)
    };

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
        .wrap_err("Test request perform error")?
        .error_for_status()
        .wrap_err("Server returned error status")?;

    // Ответ от сервера
    let response_text = response_obj
        .text()
        .await
        .wrap_err("Response body receive failed")?;
    debug!(logger, "Received from server: {response_text}");

    // Парсим
    let response_data =
        serde_json::from_str::<JsonResponse>(&response_text).wrap_err("Json parsing failed")?;

    // Вычисляем подпись от данных ответа
    let calculated_signature = {
        let mut hasher = Sha1::new();
        hasher.update(response_data.data.validation_result.as_bytes());
        hasher.update(secret_key.as_bytes());
        let hash_number = hasher.finalize();
        format!("{:x}", hash_number)
    };

    // Проверяем подпись
    eyre::ensure!(
        calculated_signature == response_data.data.validation_result_signature,
        "Response signature invalid: calculated {} != received {}",
        calculated_signature,
        response_data.data.validation_result_signature
    );

    // Парсим
    let response_data = {
        let response_json_data = base64::decode(response_data.data.validation_result).wrap_err("Base64 decode failed")?;
        let response_json_string = std::str::from_utf8(&response_json_data).wrap_err("UTF-8 parsing failed")?;
        debug!(logger, "Received json text: {response_json_string}");
        serde_json::from_str::<PurchaseStatus>(response_json_string)
            .wrap_err("Json parsing failed")?
    };

    eyre::ensure!(
        response_data.status == test.response.status,
        "Status invalid: received {} != required {}",
        response_data.status,
        test.response.status
    );

    Ok(())
}

#[tokio::main]
async fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().expect("Color eyre initialize failed");

    // Паника в красивом виде
    // human_panic::setup_panic!();

    // Аргументы приложения
    let app_arguments = AppArguments::new().expect("App arguments parsing failed");

    // Логи
    let logger = init_logs(&app_arguments);

    // Покажем параметры для отладки
    debug!(logger, "App arguments: {:?}", app_arguments);

    // Загружаем файлик конфига
    let config = Config::parse_from_file(app_arguments.config).expect("Config load failed");
    debug!(logger, "App config: {:?}", config);

    // Создаем переиспользуемый HTTP клиент
    let http_client = reqwest::ClientBuilder::new()
        .build()
        .expect("HTTP clien build failed");

    // Разворачиваем на отдельные поля
    let Config {
        api_url,
        project_name,
        secret_key,
        tests,
    } = config;

    /*
    // Идем по всем тестам и выполняем их
    const MAX_PARALLEL_COUNT: u8 = 4;
    let api_url = Arc::new(api_url);
    let mut active_futures = Vec::new();
    for (i, test) in tests.into_iter().enumerate() {
        if active_futures.len() < MAX_PARALLEL_COUNT as usize {
            // Создаем логирование для данной задачи с контекстом
            let logger = logger
                    .new(slog::o!("index" => format!("{}", i), "product" => test.purchase.product_id.clone()));

            // Клоны Arc для асинхронной задачи
            let http_client = http_client.clone();
            let api_url = api_url.clone();

            // Асинхронная задача для проверки
            let future_for_execute = async move {
                info!(logger, "Test start");
                match check_purchase(&logger, test, &http_client, &api_url).await {
                    Ok(_) => {
                        info!(logger, "Test passed");
                    }
                    Err(err) => {
                        crit!(logger, "Test failed: {err:#}");
                    }
                }
            };

            // Добавляем запинированную в куче футуру
            active_futures.push(Box::pin(future_for_execute));
        } else {
            // Ждем хоть одну завершенную задачу
            let (_, _, left_futures) = futures::future::select_all(active_futures).await;
            active_futures = left_futures;
        }
    }
    while !active_futures.is_empty() {
        let (_, _, left_futures) = futures::future::select_all(active_futures).await;
        active_futures = left_futures;
    }*/

    for (i, test) in tests.into_iter().enumerate() {
        // Создаем логирование для данной задачи с контекстом
        let logger = logger.new(
            slog::o!("index" => format!("{}", i), "product" => test.purchase.product_id.clone()),
        );

        trace!(logger, "Test start");

        match check_purchase(
            &logger,
            test,
            &http_client,
            &project_name,
            &secret_key,
            &api_url,
        )
        .await
        {
            Ok(_) => {
                info!(logger, "Test passed");
            }
            Err(err) => {
                crit!(logger, "Test failed: {err:#}");
                //std::process::exit(1);
            }
        }
    }
}
