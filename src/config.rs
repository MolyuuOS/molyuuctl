use std::fs;

use lazy_static::lazy_static;
use toml::Value;

use crate::common::structs::cell::Cell;
use crate::system::privilege;

static DEFAULT_CONFIG: &'static str = "config.toml";
pub static DEFAULT_CONFIG_DIRECTORY: &'static str = "/etc/molyuuctl";


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
            config_path.unwrap().to_string()
        } else {
            format!("{}/{}", DEFAULT_CONFIG_DIRECTORY, DEFAULT_CONFIG)
        };

        let contents = fs::read_to_string(file_path.as_str()).unwrap();
        let value = contents.parse::<Value>().unwrap();

        Self {
            path: file_path,
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
        unsafe {
            privilege::exec(|| {
                fs::write(&self.path, toml::to_string(self.value.get_mut().unwrap()).unwrap())?;
                Ok(())
            }).unwrap();
        }
    }
}