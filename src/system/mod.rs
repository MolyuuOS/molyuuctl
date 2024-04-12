use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::system::systemctl::SystemD;

mod systemctl;

pub mod privilege;

lazy_static! {
    pub static ref SYSTEMCTL: Mutex<SystemD> = Mutex::new(SystemD::new().unwrap());
}