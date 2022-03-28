mod app_arguments;

use crate::app_arguments::AppArguments;
use eyre::{ContextCompat, WrapErr};
use owo_colors::OwoColorize;
use slog::{crit, debug, Drain, Level, Logger};
use slog_async::OverflowStrategy;
use std::sync::Arc;
use validate_lib::{check_purchase, Config};

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
    let configs = {
        let mut configs = Vec::new();
        for path in app_arguments.configs.iter() {
            let config = Config::parse_from_file(path).wrap_err("Config load failed")?;
            configs.push(config);
        }
        Arc::new(configs)
    };

    // Создаем переиспользуемый HTTP клиент
    let http_client = reqwest::ClientBuilder::new()
        .tcp_keepalive(None)
        .build()
        .wrap_err("HTTP clien build failed")?;

    struct ThreadResult {
        total_finished_requests: u64,
        total_requests_duration: std::time::Duration,
    }

    let mut executors = Vec::new();
    executors.reserve(app_arguments.requests_parallel_threads as usize);
    for _ in 0..app_arguments.requests_parallel_threads {
        let logger = logger.clone();
        let configs = configs.clone();
        let http_client = http_client.clone();
        let requests_per_thread = app_arguments.requests_per_thread;

        let join = tokio::spawn(async move {
            let mut total_finished_requests = 0;
            let mut total_requests_duration = std::time::Duration::ZERO;

            while total_finished_requests < requests_per_thread {
                // Идем по каждому конфигу
                for config in configs.iter() {
                    // Идем по каждому тесту
                    for test in config.tests.iter() {
                        // Создаем логирование для данной задачи с контекстом
                        let logger =
                            logger.new(slog::o!("product" => test.purchase.product_id.clone()));

                        let begin_time = std::time::Instant::now();

                        check_purchase(&logger, &http_client, &config.project, test)
                            .await
                            .wrap_err("Request failed")?;

                        let time_spent = std::time::Instant::now().duration_since(begin_time);

                        total_requests_duration = total_requests_duration
                            .checked_add(time_spent)
                            .wrap_err("Duration overflow")?;

                        total_finished_requests += 1;
                    }
                }
            }

            Result::<ThreadResult, eyre::Error>::Ok(ThreadResult {
                total_finished_requests,
                total_requests_duration,
            })
        });

        executors.push(join);
    }

    let mut total_finished_requests = 0;
    let mut total_requests_duration = std::time::Duration::ZERO;
    for join in executors.into_iter() {
        match join.await.wrap_err("Request spawn join failed")? {
            Ok(thread_stats) => {
                total_finished_requests += thread_stats.total_finished_requests;
                total_requests_duration += thread_stats.total_requests_duration;
            }
            Err(err) => {
                crit!(logger, "Request execution failed");
                return Err(err);
            }
        }
    }

    let average_msec = total_requests_duration.as_millis() / total_finished_requests as u128;
    println!("Average time per request: {} mSec", average_msec.green());

    Ok(())
}
