use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::tools::systemctl::SystemD;

mod systemctl;
pub mod cleanup;
pub mod privilege;
pub mod cell;

lazy_static! {
    pub static ref SYSTEMCTL: Mutex<SystemD> = Mutex::new(SystemD::new().unwrap());
}