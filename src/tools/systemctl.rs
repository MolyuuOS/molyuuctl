use std::error::Error;
use std::time::Duration;

use dbus::blocking::{Connection, Proxy};

pub struct SystemD {
    conn: Connection,
}

impl SystemD {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            conn: Connection::new_system()?
        })
    }

    fn get_proxy(&self) -> Result<Proxy<'_, &'_ Connection>, Box<dyn Error>> {
        Ok(self.conn.with_proxy("org.freedesktop.systemd1", "/org/freedesktop/systemd1", Duration::from_millis(5000)))
    }

    pub fn reset_failed_unit(&self, unit: &str) -> Result<(), Box<dyn Error>> {
        self.get_proxy()?.method_call("org.freedesktop.systemd1.Manager", "ResetFailedUnit", ("lightdm", ))?;
        Ok(())
    }

    pub fn restart_unit(&self, unit: &str) -> Result<(), Box<dyn Error>> {
        let (job, ): (String, ) = self.get_proxy()?.method_call("org.freedesktop.systemd1.Manager", "RestartUnit", ("lightdm", "replace", ))?;
        Ok(())
    }
}