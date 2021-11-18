use crate::domain::config::Config;

pub struct Daemon {
    config: Config,
}

impl Daemon {
    pub fn new(config: Config) -> Self {
        Daemon { config }
    }

    pub fn run(self) {
        println!("{:?}", self.config);
        unimplemented!();
    }
}
