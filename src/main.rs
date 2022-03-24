mod app_arguments;
mod app_config;
mod helpers;

use crate::{
    app_arguments::AppArguments,
    app_config::{Config, PurchaseData},
};
use serde::{Deserialize, Serialize};
use slog::{debug, info, Drain, Logger};
use slog_async::OverflowStrategy;
// use sloggers::{
//     terminal::{Destination, TerminalLoggerBuilder},
//     types::Severity,
//     Build,
// };
// use std::env::{self};

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn init_logs() -> Logger {
    /*const LOG_VAR: &str = "RUST_LOG";
    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "info");
    }
    pretty_env_logger::try_init().expect("Logger setup failed");*/

    let drain = slog_term::TermDecorator::new().stdout().build();
    let drain = slog_term::FullFormat::new(drain)
        .use_file_location()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain)
        .overflow_strategy(OverflowStrategy::Block)
        .build()
        .fuse();
    slog::Logger::root(drain, slog::o!())

    // let _log_guard = slog_scope::set_global_logger(logger);
    // let _guard = slog_stdlog::init().expect("Slog as log backend");
}

#[tokio::main]
async fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().expect("Color eyre initialize failed");

    // Паника в красивом виде
    // human_panic::setup_panic!();

    // Логи
    let logger = init_logs();

    // Аргументы приложения
    let app_arguments = AppArguments::new().expect("App arguments parsing failed");
    debug!(logger, "App arguments: {:?}", app_arguments);

    // Загружаем файлик конфига
    let config = Config::parse_from_file(app_arguments.config).expect("Config load failed");
    debug!(logger, "App config: {:?}", config);

    // Создаем переиспользуемый HTTP клиент
    let http_client = reqwest::ClientBuilder::new()
        .build()
        .expect("HTTP clien build failed");

    // Идем по всем тестам и выполняем их
    for test in config.tests.into_iter() {
        // TODO: Убрать клонирование
        let logger = logger.new(slog::o!("product" => test.purchase.product_id.clone()));
        info!(logger, "Test start"; "test" => "test");

        #[derive(Debug, Serialize)]
        struct JsonBody {
            purchase_data: PurchaseData,
        }

        // Выполняем запрос
        let response_obj = http_client
            .post(config.api_url.clone())
            .json(&JsonBody {
                purchase_data: test.purchase,
            })
            .send()
            .await
            .expect("Test request perform error")
            .error_for_status()
            .expect("Server returned error status");

        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct PurchaseStatus {
            status: String,
            description: Option<String>,
            payload: Option<Vec<String>>,
        }
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct JsonResponse {
            message: Option<String>,
            #[serde(with = "chrono::serde::ts_seconds")]
            timestamp: chrono::DateTime<chrono::Utc>,
            datetime: String,
            data: PurchaseStatus,
        }

        let response_text = response_obj
            .text()
            .await
            .expect("Response body receive failed");
        debug!(logger, "Received from server"; "response_text" => &response_text);

        let response_data =
            serde_json::from_str::<JsonResponse>(&response_text).expect("Json parsing failed");
        assert_eq!(
            response_data.data.status, test.response.status,
            "Invalid test response status"
        );

        // drop(logger);
        // drop(test);
    }
}
