use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use std::string::String;

use ini::Ini;
use toml::{Table, Value};

use crate::config;
use crate::config::helper::GLOBAL_CONFIG;
use crate::login::manager::get_current_manager;
use crate::session::protocol::Protocol;

static SYSTEM_XSESSIONS_PATH: &'static str = "/usr/share/xsessions";
static SYSTEM_WAYLAND_SESSIONS_PATH: &'static str = "/usr/share/wayland-sessions";

pub struct Session {
    reg_name: String,
    real_name: String,
    logout_command: Option<String>,
    protocol: Protocol,
}

impl Session {
    pub fn new(reg_name: String, real_name: String, logout_command: Option<String>, protocol: Option<Protocol>) -> Result<Self, Box<dyn Error>> {
        let detected_protocol = match protocol {
            Some(Protocol::X11) => {
                if !Path::new(format!("{SYSTEM_XSESSIONS_PATH}/{reg_name}.desktop").as_str()).exists() {
                    return Err(Box::from("Specific session not found!"));
                }
                Protocol::X11
            }
            Some(Protocol::Wayland) => {
                if !Path::new(format!("{SYSTEM_WAYLAND_SESSIONS_PATH}/{reg_name}.desktop").as_str()).exists() {
                    return Err(Box::from("Specific session not found!"));
                }
                Protocol::Wayland
            }
            _ => {
                let detected_protocol = Self::find_session_in_system(real_name.as_str());
                if detected_protocol.is_err() {
                    return Err(Box::from("Specific session not found!"));
                }
                detected_protocol?
            }
        };

        Ok(Self {
            reg_name,
            real_name,
            logout_command,
            protocol: detected_protocol,
        })
    }

    pub fn from_config(session_name: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        let session_reg_name = if session_name.is_none() {
            String::from(session_info["default"].as_str().unwrap())
        } else {
            String::from(session_name.unwrap())
        };
        if session_info.get(session_reg_name.as_str()).is_none() {
            return Err(Box::from("Session Not Found"));
        }

        let mut session_real_name = String::new();
        let mut session_logout_command = None;
        let mut session_protocol = None;
        for session in session_info {
            if session.0 == session_reg_name.as_str() {
                session_real_name = String::from(session.1["session"].as_str().unwrap());
                let try_get_protocol = session.1.get("protocol");
                let try_get_logoutcmd = session.1.get("logoutcmd");
                if try_get_protocol.is_none() {
                    session_protocol = Some(Self::find_session_in_system(session_real_name.as_str())?)
                } else {
                    session_protocol = match try_get_protocol.unwrap().as_str() {
                        Some("x11") => Some(Protocol::X11),
                        Some("wayland") => Some(Protocol::Wayland),
                        _ => return Err(Box::from("Unknown protocol"))
                    }
                }
                if try_get_logoutcmd.is_some() {
                    session_logout_command = Some(String::from(try_get_logoutcmd.unwrap().as_str().unwrap()));
                }
                break;
            }
        };

        Ok(Self {
            reg_name: String::from(session_reg_name),
            real_name: String::from(session_real_name),
            logout_command: session_logout_command,
            protocol: session_protocol.unwrap(),
        })
    }

    pub fn find_session_in_system(real_session_name: &str) -> Result<Protocol, Box<dyn Error>> {
        let protocol = if Path::new(format!("{SYSTEM_XSESSIONS_PATH}/{real_session_name}.desktop").as_str()).exists() {
            Protocol::X11
        } else if Path::new(format!("{SYSTEM_WAYLAND_SESSIONS_PATH}/{real_session_name}.desktop").as_str()).exists() {
            Protocol::Wayland
        } else {
            return Err(Box::from("Session Not Found"));
        };

        Ok(protocol)
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let session_file_location = if self.protocol == Protocol::X11 {
            SYSTEM_XSESSIONS_PATH
        } else if self.protocol == Protocol::Wayland {
            SYSTEM_WAYLAND_SESSIONS_PATH
        } else {
            return Err(Box::from("Unknown protocol"));
        };

        let session_real_name = self.real_name.as_str();
        let session_file = Ini::load_from_file(format!("{session_file_location}/{session_real_name}.desktop"))?;
        let desktop_section = session_file.section(Some("Desktop Entry")).unwrap();
        let command = desktop_section.get("Exec").unwrap();
        println!("Target Session: {}", desktop_section.get("Name").unwrap());
        println!("Executing Session Command: {}", command);
        Command::new("/bin/bash")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to launch session");
        Ok(())
    }

    pub fn start_oneshot_or_default_session() -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        let oneshot_session = session_info.get("oneshot_session");
        let oneshot_started = session_info.get("oneshot_started");
        if oneshot_session.is_some() && oneshot_started.is_some() && !oneshot_started.unwrap().as_bool().unwrap() {
            let session_to_start = String::from(oneshot_session.unwrap().as_str().unwrap());
            session_info["oneshot_started"] = Value::Boolean(true);
            GLOBAL_CONFIG.get_mut().unwrap().save_config();
            Self::from_config(Some(session_to_start.as_str())).unwrap().start()?;
        } else {
            Self::from_config(None).unwrap().start()?;
        }

        // Update Login Manager config
        get_current_manager()?.save_config()?;
        Ok(())
    }

    pub fn logout(&self) -> Result<(), Box<dyn Error>> {
        if self.logout_command.is_none() {
            return Err(Box::from("No logout command is set, cannot logout"));
        }
        Command::new("/bin/bash")
            .arg("-c")
            .arg(self.logout_command.as_ref().unwrap().as_str())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to logout session");
        Ok(())
    }

    pub fn rename(&mut self, new_name: &str) -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        if session_info.get(new_name).is_some() {
            return Err(Box::from("Specific session already exist"));
        }

        let old_name = self.reg_name.clone();
        let current_session_info = session_info.get(self.reg_name.as_str()).unwrap();
        session_info.insert(String::from(new_name), current_session_info.clone());
        session_info.remove(&self.reg_name);
        self.reg_name = String::from(new_name);
        if session_info.get("default").unwrap().as_str() == Some(old_name.as_str()) {
            session_info["default"] = Value::String(self.reg_name.clone());
        }
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    pub fn remove(&self) -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        if session_info.get("default").unwrap().as_str() == Some(self.reg_name.as_str()) {
            return Err(Box::from("Cannot remove default session"));
        }
        session_info.remove(&self.reg_name);
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    pub fn register(&mut self) -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        if session_info.get(self.reg_name.as_str()).is_some() {
            return Err(Box::from("Session already registered or name is occupied"));
        }

        let protocol_str = if self.protocol == Protocol::X11 {
            "x11"
        } else if self.protocol == Protocol::Wayland {
            "wayland"
        } else {
            return Err(Box::from("Unknown protocol"));
        };

        let mut new_table = Table::new();
        new_table.insert(String::from("session"), Value::String(self.real_name.clone()));
        new_table.insert(String::from("protocol"), Value::String(String::from(protocol_str)));
        if self.logout_command.is_some() {
            new_table.insert(String::from("logoutcmd"), Value::String(self.logout_command.as_ref().unwrap().clone()));
        };
        session_info.insert(String::from(&self.reg_name), Value::Table(new_table));
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    pub fn set_logout_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        let current_session_section = session_info.get_mut(self.reg_name.as_str()).unwrap().as_table_mut().unwrap();
        current_session_section.insert(String::from("logoutcmd"), toml::Value::String(String::from(command)));
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    pub fn set_as_default(&self) -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        session_info["default"] = toml::Value::String(self.reg_name.clone());
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    pub fn set_start_oneshot(&self) -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        if session_info.get("oneshot_session").is_none() {
            session_info.insert(String::from("oneshot_session"), Value::String(self.reg_name.clone()));
        } else {
            session_info["oneshot_session"] = Value::String(self.reg_name.clone());
        }

        if session_info.get("oneshot_started").is_none() {
            session_info.insert(String::from("oneshot_started"), Value::Boolean(false));
        } else {
            session_info["oneshot_started"] = Value::Boolean(false);
        }
        GLOBAL_CONFIG.get_mut().unwrap().save_config();

        // We need to set to correct protocol in Login Manager for session change
        get_current_manager()?.save_config()?;
        Ok(())
    }

    pub fn get_protocol(&self) -> Protocol {
        self.protocol
    }

    pub fn get_default_session() -> Result<Self, Box<dyn Error>> {
        Ok(Self::from_config(None).unwrap())
    }

    pub fn get_oneshot_session() -> Result<Option<Self>, Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table().unwrap();
        let oneshot_session = session_info.get("oneshot_session");
        let oneshot_started = session_info.get("oneshot_started");
        if oneshot_session.is_some() && oneshot_started.is_some() && !oneshot_started.unwrap().as_bool().unwrap() {
            return Ok(Some(Self::from_config(Some(oneshot_session.unwrap().as_str().unwrap()))?));
        }
        Ok(None)
    }
}