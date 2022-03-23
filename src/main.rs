

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() -> Result<(), eyre::Error> {
    Ok(())
}

#[tokio::main]
async fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().expect("Color eyre initialize failed");

    // Логи
    initialize_logs().expect("Logs init");

    // Аргументы приложения
    // let app_arguments = AppArguments::from_args();
    // debug!("App arguments: {:?}", app_arguments);

    // Проверка аргументов приложения
    // app_arguments.validate_arguments().expect("Invalid argument");

    // Загружаем файлик конфига
    // let config = Config::parse_from_file(app_arguments.config).expect("Config load failed");

}
