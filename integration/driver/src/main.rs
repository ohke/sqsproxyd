use aws_sdk_sqs::{Client, Endpoint};
use http::Uri;
use serde::Deserialize;
use structopt::StructOpt;
use tokio::time::{sleep, Duration};
use url::Url;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, env = "SQSPROXYD_AWS_ENDPOINT")]
    pub aws_endpoint: Uri,
    #[structopt(long, env = "SQSPROXYD_SQS_URL")]
    pub sqs_url: Url,
    #[structopt(long, env = "SQSPROXYD_OUTPUT_SQS_URL")]
    pub output_sqs_url: Url,
}

impl Opt {
    pub fn new() -> Self {
        Self::from_args()
    }
}

#[derive(Debug, Deserialize)]
struct Body {
    result: i32,
}

#[tokio::main]
async fn main() {
    let opt = Opt::new();

    let sqs_config = aws_sdk_sqs::config::Builder::from(&aws_config::from_env().load().await)
        .endpoint_resolver(Endpoint::immutable(opt.aws_endpoint.clone()))
        .build();
    let client = Client::from_conf(sqs_config);

    println!("Waiting for SQS container start up...");
    sleep(Duration::from_secs(5)).await;

    // enqueue to sqs.
    client
        .send_message()
        .queue_url(&opt.sqs_url.to_string())
        .message_body("{\"x\":1,\"y\":2}")
        .send()
        .await
        .unwrap();

    // dequeue from output sqs.
    for i in 0..30 {
        let output = client
            .receive_message()
            .queue_url(&opt.output_sqs_url.to_string())
            .send()
            .await
            .unwrap();
        match output.messages {
            None => {
                println!("[{}] None of messages. Waiting...", i);
                sleep(Duration::from_secs(1)).await;
            },
            Some(mut messages) => {
                match messages.len() {
                    0 => {
                        println!("[{}] Zero messages. Waiting...", i);
                        sleep(Duration::from_secs(1)).await;
                    },
                    1 => {
                        let body: Body =
                            serde_json::from_str(messages.pop().unwrap().body.unwrap().as_str())
                                .unwrap();
                        assert_eq!(body.result, 3);
                        return;
                    }
                    _ => panic!("{}", format!("output.messages.len() > 1.")),
                }
            }
        }
    }

    panic!(
        "{}",
        format!("failed to received message from output sqs. {:?}", opt)
    );
}
