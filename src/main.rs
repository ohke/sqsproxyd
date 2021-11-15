mod domain;

use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

use domain::config::Config;

#[derive(StructOpt, Debug)]
#[structopt(name = "sqsproxyd")]
pub struct Arg {
    #[structopt(short, long, parse(from_os_str))]
    env: Option<PathBuf>,
}

fn main() -> Result<()> {
    // get configuration parameters
    let arg = Arg::from_args();
    if let Some(v) = arg.env {
        dotenv::from_filename(v).expect("Not found env file.");
    } else {
        dotenv::dotenv().ok();
    }

    let config = Config::new()?;

    println!("{:#?}", config);

    Ok(())
}
