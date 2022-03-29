mod app_arguments;

use crate::app_arguments::AppArguments;
use eyre::WrapErr;
use owo_colors::OwoColorize;
use reqwest::Client;
use slog::{debug, trace, Drain, Level, Logger};
use slog_async::OverflowStrategy;
use validate_lib::{check_purchase, Config};
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

/// Выполняем обработку тестовых платежей
async fn execute_tests(logger: &Logger, http_client: &Client, config: &Config) {
    // Разворачиваем на отдельные поля
    let Config { project, tests } = config;

    for (i, test) in tests.iter().enumerate() {
        let index = i+1;

        // Создаем логирование для данной задачи с контекстом
        let logger = logger.new(
            slog::o!("index" => format!("{}", index), "product" => test.purchase.product_id.clone()),
        );

        trace!(logger, "Test start");

        match check_purchase(&logger, http_client, project, test).await {
            Ok(_) => {
                println!(r#"{}: test number "{}", order_id: "{}""#, "Test passed".green(), index, test.purchase.order_id);
            }
            Err(err) => {
                eprintln!(r#"{}: test number "{}", order_id: "{}", err: "{err:#}""#, "Test failed".red(), index, test.purchase.order_id);
                std::process::exit(1);
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

    // Загружаем файлики конфига
    let configs = {
        let mut configs = Vec::new();
        for path in app_arguments.configs.iter() {
            let config = Config::parse_from_file(path).wrap_err("Config load failed")?;
            configs.push(config);
        }
        configs
    };
    debug!(logger, "App configs: {:?}", configs);

    // Создаем переиспользуемый HTTP клиент
    let http_client = reqwest::ClientBuilder::new()
        .build()
        .wrap_err("HTTP clien build failed")?;

    // Идем по списку конфигов и прогоняем каждый
    for config in configs.iter() {
        println!("Begin project: {}", config.project.name.blue());
        execute_tests(&logger, &http_client, config).await;
    }

    Ok(())
}
