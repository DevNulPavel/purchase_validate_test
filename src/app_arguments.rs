use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Application config file path
    #[structopt(short, long, parse(from_os_str), env = "PURCHASE_VALIDATE_CONFIG_FILE_PATH")]
    pub config: PathBuf,
}

impl AppArguments {
    pub fn new() -> Result<Self, &'static str> {
        let args = AppArguments::from_args();
        args.validate_arguments()?;
        Ok(args)
    }

    /// Выполняем валидацию переданных аргументов приложения
    fn validate_arguments(&self) -> Result<(), &'static str> {
        macro_rules! validate_argument {
            ($argument: expr, $desc: literal) => {
                if $argument == false {
                    return Err($desc);
                }
            };
        }

        validate_argument!(self.config.exists(), "Config file does not exist");
        validate_argument!(self.config.is_file(), "Config file is not a file");
        Ok(())
    }
}
