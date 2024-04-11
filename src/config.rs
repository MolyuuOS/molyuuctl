use std::fs;
use std::str::FromStr;

use lazy_static::lazy_static;
use toml::Value;

use crate::tools::cell::Cell;
use crate::tools::privilege;

static DEFAULT_CONFIG_PATH: &'static str = "/etc/molyuuctl/config.toml";

lazy_static! {
    pub static ref GLOBAL_CONFIG: Cell<Configuration> = Cell::default();
}

pub struct Configuration {
    path: String,
    value: Cell<Value>,
}

impl Configuration {
    fn new(config_path: Option<&str>) -> Self {
        let file_path = if config_path.is_some() {
            config_path.unwrap()
        } else {
            DEFAULT_CONFIG_PATH
        };

        let contents = fs::read_to_string(file_path).unwrap();
        let value = contents.parse::<Value>().unwrap();

        Self {
            path: String::from_str(file_path).unwrap(),
            value: Cell::new(value),
        }
    }

    pub fn init(config_path: Option<&str>) {
        GLOBAL_CONFIG.init(Self::new(config_path)).unwrap();
    }

    pub fn get(&mut self, config_name: &str) -> &mut Value {
        &mut self.value.get_mut().unwrap()[config_name]
    }

    pub fn save_config(&mut self) {
        privilege::write(toml::to_string(self.value.get_mut().unwrap()).unwrap().as_str(), &self.path).unwrap();
    }
}