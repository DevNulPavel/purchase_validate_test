mod app_arguments;
mod app_config;

use crate::{app_arguments::AppArguments, app_config::Config};
use eyre::WrapErr;
use reqwest::Client;
use slog::{crit, debug, info, trace, Drain, Level, Logger};
use slog_async::OverflowStrategy;
use validate_lib::check_purchase;
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

/*async fn execute_tests(logger: Logger, http_client: Client, config: Config) {
    // Разворачиваем на отдельные поля
    let Config {
        api_url,
        project_name,
        secret_key,
        tests,
    } = config;

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
    }
}*/

/// Выполняем обработку тестовых платежей
async fn execute_tests(logger: Logger, http_client: Client, config: Config) {
    // Разворачиваем на отдельные поля
    let Config {
        api_url,
        project_name,
        secret_key,
        tests,
    } = config;

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

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Бектрейсы в ошибках
    color_eyre::install().wrap_err("Color eyre initialize failed")?;

    // Паника в красивом виде
    // human_panic::setup_panic!();

    // Аргументы приложения
    let app_arguments = AppArguments::new().wrap_err("App arguments parsing failed")?;

    // Логи
    let logger = init_logs(&app_arguments);

    // Покажем параметры для отладки
    debug!(logger, "App arguments: {:?}", app_arguments);

    // Загружаем файлик конфига
    let config = Config::parse_from_file(app_arguments.config).wrap_err("Config load failed")?;
    debug!(logger, "App config: {:?}", config);

    // Создаем переиспользуемый HTTP клиент
    let http_client = reqwest::ClientBuilder::new()
        .build()
        .wrap_err("HTTP clien build failed")?;

    execute_tests(logger, http_client, config).await;

    Ok(())
}
