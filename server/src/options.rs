use std::net::IpAddr;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    /// IP-address to listen for incoming requests on.
    #[structopt(short, long, env = "IP_ADDR", default_value = "0.0.0.0")]
    pub address: IpAddr,

    /// Port to listen for incoming requests on.
    #[structopt(short, long, env = "PORT", default_value = "8081")]
    pub port: u16,

    /// Path to the directory containing all pages.
    #[structopt(short = "d", long = "pages", default_value = "pages")]
    pub pages_directory: PathBuf,

    /// Path to the directory containing all templates.
    #[structopt(short = "t", long = "templates", default_value = "templates")]
    pub templates_directory: PathBuf,

    /// Upgade pages on any changes to the 'pages' and 'templates' directories.
    #[structopt(short, long)]
    pub watch: bool,

    /// Number of seconds to wait to update after files have changed
    #[structopt(long, default_value = "1")]
    pub delay: f32,
}

