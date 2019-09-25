use std::net::IpAddr;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(short, long, env = "PORT", default_value = "8081")]
    pub port: u16,
    #[structopt(short, long, env = "IP_ADDR", default_value = "0.0.0.0")]
    pub address: IpAddr,

    #[structopt(short = "d", long = "pages", default_value = "pages")]
    pub pages_directory: PathBuf,
    #[structopt(short = "t", long = "templates", default_value = "templates")]
    pub templates_directory: PathBuf,

    #[structopt(short, long)]
    pub watch: bool,
}

