use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "bird-lg-server", version, about = "BIRD looking glass server")]
pub struct Cli {
    #[arg(short, long, value_name = "FILE", default_value = "config.json")]
    pub config: String,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
