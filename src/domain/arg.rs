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
    pub webhook_health_check_url: Option<Url>,
    #[structopt(long)]
    pub content_type: Option<String>,
}
