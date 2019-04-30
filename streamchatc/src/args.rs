use super::config::{Config, Fringe};
use configurable::Configurable;
use gumdrop::Options;

#[derive(Debug, Options)]
pub struct Args {
    #[options(help = "show this help message")]
    pub help: bool,

    #[options(help = "left fringe to use", meta = "STRING")]
    pub left: Option<String>,

    #[options(help = "left fringe color", no_short, meta = "#RRGGBB")]
    pub left_color: Option<String>,

    #[options(help = "right fringe to use", meta = "STRING")]
    pub right: Option<String>,

    #[options(help = "right fringe color", no_short, meta = "#RRGGBB")]
    pub right_color: Option<String>,

    #[options(help = "address of the streamchatd instance", meta = "ADDR")]
    pub address: Option<String>,

    #[options(help = "maximum number of messages to buffer", short = "n", meta = "N")]
    pub buffer_max: Option<usize>,

    #[options(help = "maximum width of nicknames", short = "m", meta = "N")]
    pub nick_max: Option<usize>,

    #[options(help = "print the configuration path", no_short)]
    pub config: bool,
}

impl Args {
    pub fn load_or_config() -> Config {
        use configurable::LoadState::*;
        let mut config = match Config::load_or_default() {
            Ok(Loaded(config)) => config,
            Ok(Default(config)) => {
                eprintln!("creating a default config.");
                eprintln!(
                    "look for it at: {}",
                    Config::path().unwrap().to_string_lossy()
                );
                config.save().unwrap();
                std::process::exit(2)
            }
            Err(err) => {
                eprintln!("cannot load config: {}", err);
                std::process::exit(1)
            }
        };

        let args = Args::parse_args_default_or_exit();
        if args.config {
            eprintln!("{}", Config::path().unwrap().to_string_lossy());
            std::process::exit(0);
        }

        macro_rules! replace {
            ($left:expr, $right:expr) => {{
                if let Some(left) = $left {
                    $right = left
                }
            }};
        }

        match (args.left, args.left_color) {
            (Some(fringe), Some(color)) => config.left_fringe = Fringe { fringe, color },
            (Some(fringe), None) => config.left_fringe.fringe = fringe,
            (None, Some(color)) => config.left_fringe.color = color,
            _ => {}
        }

        match (args.right, args.right_color) {
            (Some(fringe), Some(color)) => config.right_fringe = Fringe { fringe, color },
            (Some(fringe), None) => config.right_fringe.fringe = fringe,
            (None, Some(color)) => config.right_fringe.color = color,
            _ => {}
        }

        replace!(args.address, config.address);
        replace!(args.buffer_max, config.buffer_max);
        replace!(args.nick_max, config.nick_max);

        config
    }
}
