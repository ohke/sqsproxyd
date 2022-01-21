# sqsproxyd
**sqsproxyd** is SQS proxy daemon.

This is an application that imitates [SQS daemon](https://docs.aws.amazon.com/elasticbeanstalk/latest/dg/using-features-managing-env-tiers.html) provided in the AWS Elastic Beanstalk worker environment.
In addition, it has the ability to enqueue a response to another SQS.

## Features
- Dequeue a message from SQS and make a POST request to the specified Webhook API.
- If the Webhook API returns a success response (HTTP status: 2**), removes the message from the SQS.
- [Option] In addition, if an output SQS is set, the Webhook API success response body be enqueued to that SQS as a message.

## Usage

### Execution

#### Command-line (binary)
The binaries can be downloaded [here](https://github.com/ohke/sqsproxyd/releases).

```bash
$ sqsproxyd \
  --sqs-url https://sqs.us-west-1.amazonaws.com/123456789012/sqsproxyd-sqs \
  --webhook-url http://localhost:4000/api 
```

#### Docker container
The sqsproxyd container image can also be pulled from Docker Hub.

```bash
$ docker pull ohke/sqsproxyd
$ docker run ohke/sqsproxyd \
  --sqs-url https://sqs.us-west-1.amazonaws.com/123456789012/sqsproxyd-sqs \
  --webhook-url http://localhost:4000/api
```

### Configuration
Either method can be used to pass parameters. If a value exists for both, command-line arguments take precedence.

- Environment variables
- Command-line arguments

#### Parameters

| Command-line argument | Environment variable | Required | Default | Description |
| -- | -- | -- | -- | -- | 
| --aws-access-key-id | AWS_ACCESS_KEY_ID | no | - | |
| --aws-secret-access-key | AWS_SECRET_ACCESS_KEY | no | - | |
| --aws-session-token | AWS_SESSION_TOKEN | no | - | |
| --aws-region | SQSPROXYD_AWS_REGION or AWS_DEFAULT_REGION | no | - | |
| --aws-endpoint | SQSPROXYD_AWS_ENDPOINT | no | - | |
| --sqs-url | SQSPROXYD_SQS_URL | yes | - |  |
| --webhook-url | SQSPROXYD_WEBHOOK_URL | yes | - |  |
| --output-sqs-url | SQSPROXYD_OUTPUT_SQS_URL | no | - | |
| --worker-concurrency | SQSPROXYD_WORKER_CONCURRENCY | no | 1 | |
| --connection-timeout | SQSPROXYD_CONNECTION_TIMEOUT | no | 30 | |
| --max-number-of-messages | SQSPROXYD_MAX_NUMBER_OF_MESSAGES | no | 1 | |
| --sleep-seconds | SQSPROXYD_SLEEP_SECONDS | no | 1 | |
| --webhook-healthcheck-url | SQSPROXYD_WEBHOOK_HEALTHCHECK_URL | no | - | |
| --webhook-healthcheck-interval-seconds | SQSPROXYD_WEBHOOK_HEALTHCHECK_INTERVAL_SECONDS | no | 1 | |
| --content-type | SQSPROXYD_CONTENT_TYPE | no | `application/json` | |
| --rust-log | SQSPROXYD_RUST_LOG | no | `WARN` | |

## Development
