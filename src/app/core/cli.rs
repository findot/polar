use std::path::{Path, PathBuf};
use clap::Parser;

// TODO
#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value = "/etc/lance/conf.toml")]
    configuration: Path,

    database_host: String,
    database_port: u16,
    database_user: String,
    database_password: String,
    database_schema: String
}