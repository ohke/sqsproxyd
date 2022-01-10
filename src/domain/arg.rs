use structopt::StructOpt;
use url::Url;

#[derive(Debug, StructOpt)]
#[structopt(name = "sqsproxyd")]
pub struct Arg {
    #[structopt(long)]
    pub aws_access_key_id: Option<String>,
    #[structopt(long)]
    pub aws_secret_access_key: Option<String>,
    #[structopt(long)]
    pub aws_session_token: Option<String>,
    #[structopt(long)]
    pub aws_region: Option<String>,
    #[structopt(long)]
    pub aws_endpoint: Option<String>,
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
    pub webhook_health_check_interval_seconds: Option<u64>,
    #[structopt(long)]
    pub content_type: Option<String>,
}
