use structopt::StructOpt;
use url::Url;

#[derive(Debug, StructOpt)]
#[structopt(name = "sqsproxyd")]
pub struct Arg {
    #[structopt(long)]
    pub sqs_url: Option<Url>,
    #[structopt(long)]
    pub webhook_url: Option<Url>,
    #[structopt(long)]
    pub output_sqs_url: Option<Url>,
    #[structopt(long)]
    pub worker_concurrency: Option<usize>,
    #[structopt(long)]
    pub connection_timeout: Option<u64>,
    #[structopt(long)]
    pub max_number_of_messages: Option<usize>,
    #[structopt(long)]
    pub sleep_seconds: Option<u64>,
    #[structopt(long)]
    pub webhook_health_check_path: Option<String>,
}

#[cfg(test)]
impl Arg {
    pub fn new_empty() -> Self {
        Arg {
            sqs_url: None,
            webhook_url: None,
            output_sqs_url: None,
            worker_concurrency: None,
            connection_timeout: None,
            max_number_of_messages: None,
            sleep_seconds: None,
            webhook_health_check_path: None,
        }
    }
}
