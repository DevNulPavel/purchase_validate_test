use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Test configs file paths
    #[structopt(short, long, parse(from_os_str))]
    pub configs: Vec<PathBuf>,

    /// Number threads for requests
    #[structopt(short, long)]
    pub requests_parallel_threads: u32,

    /// Requests count per thread
    #[structopt(short, long)]
    pub requests_per_thread: u64,

    /// Log level
    #[structopt(short, parse(from_occurrences))]
    pub verbose: u8,
}

impl AppArguments {
    pub fn new() -> Result<Self, eyre::Error> {
        let args = AppArguments::from_args();
        args.validate_arguments()?;
        Ok(args)
    }

    /// Выполняем валидацию переданных аргументов приложения
    fn validate_arguments(&self) -> Result<(), eyre::Error> {
        macro_rules! validate_argument {
            ($argument: expr, $desc: literal) => {
                if ($argument) == false {
                    return Err(eyre::eyre!($desc));
                }
            };
        }

        validate_argument!(!self.configs.is_empty(), "Configs array cannot be empty");

        for config in self.configs.iter() {
            validate_argument!(config.exists(), "Config file does not exist");
            validate_argument!(config.is_file(), "Config file is not a file");
        }

        validate_argument!(self.verbose < 3, "Verbose level must be in range [0; 2]");

        validate_argument!(
            self.requests_parallel_threads > 0,
            "Requests threads cannot be zero"
        );
        validate_argument!(
            self.requests_per_thread > 0,
            "Requests per thread count cannot be zero"
        );

        Ok(())
    }
}
