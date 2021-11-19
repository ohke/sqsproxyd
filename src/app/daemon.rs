use crate::domain::config::Config;
use crate::infra::sqs::Sqs;

pub struct Daemon {
    config: Config,
    _sqs: Box<dyn Sqs>,
}

impl Daemon {
    pub fn new(config: Config, sqs: Box<dyn Sqs>) -> Self {
        Daemon { config, _sqs: sqs }
    }

    pub fn run(self) {
        println!("{:?}", self.config);
        unimplemented!();
    }
}
