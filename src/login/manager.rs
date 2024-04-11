use std::error::Error;
use std::io::BufWriter;
use std::path::Path;

use ini::Ini;
use toml::Value;

use crate::config::GLOBAL_CONFIG;
use crate::session::controller::Session;
use crate::session::protocol::Protocol;
use crate::tools::{privilege, SYSTEMCTL};

pub static MOLYUU_REDIRECT_SESSION_PREFIX: &'static str = "molyuu-redirect";
static LIGHTDM_CUSTOM_CONFIG_PATH: &'static str = "/etc/lightdm/lightdm.conf.d/10-molyuud-session.conf";
static SDDM_CUSTOM_CONFIG_PATH: &'static str = "/etc/sddm.conf.d/molyuuctl.conf";


pub enum SupportedManager {
    LightDM,
    SDDM,
}

#[derive(Debug, Clone)]
pub struct ManagerMetadata {
    pub systemd_unit: String,
    pub config_path: String,
    pub autologin_section_name: String,
    pub autologin_session_key_name: String,
    pub autologin_user_key_name: String,
}

impl ManagerMetadata {
    pub fn build_for_supported_manager(manager: SupportedManager) -> Self {
        match manager {
            SupportedManager::LightDM => {
                Self {
                    systemd_unit: "lightdm".to_string(),
                    config_path: LIGHTDM_CUSTOM_CONFIG_PATH.to_string(),
                    autologin_section_name: "Seat:*".to_string(),
                    autologin_session_key_name: "autologin-session".to_string(),
                    autologin_user_key_name: "autologin-user".to_string(),
                }
            }
            SupportedManager::SDDM => {
                Self {
                    systemd_unit: "sddm".to_string(),
                    config_path: SDDM_CUSTOM_CONFIG_PATH.to_string(),
                    autologin_section_name: "Autologin".to_string(),
                    autologin_session_key_name: "Session".to_string(),
                    autologin_user_key_name: "User".to_string(),
                }
            }
        }
    }
}

pub struct ManagerBuilder(ManagerMetadata);

impl ManagerBuilder {
    pub fn new() -> Self {
        Self(ManagerMetadata {
            systemd_unit: "".to_string(),
            config_path: "".to_string(),
            autologin_section_name: "".to_string(),
            autologin_session_key_name: "".to_string(),
            autologin_user_key_name: "".to_string(),
        })
    }

    pub fn use_manager(mut self, manager: SupportedManager) -> Self {
        self.0 = ManagerMetadata::build_for_supported_manager(manager);
        self
    }

    pub fn systemd_unit(mut self, systemd_unit: &str) -> Self {
        self.0.systemd_unit = systemd_unit.to_string();
        self
    }

    pub fn use_config(mut self, config_path: &str) -> Self {
        self.0.config_path = config_path.to_string();
        self
    }

    pub fn autologin_section(mut self, section_name: &str) -> Self {
        self.0.autologin_section_name = section_name.to_string();
        self
    }

    pub fn session_key(mut self, session_key: &str) -> Self {
        self.0.autologin_session_key_name = session_key.to_string();
        self
    }

    pub fn user_key(mut self, user_key: &str) -> Self {
        self.0.autologin_user_key_name = user_key.to_string();
        self
    }

    pub fn build(&self) -> Result<Manager, Box<dyn Error>> {
        Ok(Manager::new(self.0.clone())?)
    }
}

pub struct Manager {
    autologin: bool,
    session_type: Protocol,
    login_user: Option<String>,
    metadata: ManagerMetadata,
}

impl Manager {
    /// Constructs a new instance of a Manager based on the provided metadata.
    ///
    /// This function initializes a Manager instance with the given metadata. It checks if the
    /// configuration file specified in the metadata exists. If it does, it loads the configuration
    /// file and checks for an autologin section. If an autologin section is found, it extracts the
    /// autologin session and user information. Otherwise, it initializes the Manager instance
    /// with default values.
    ///
    /// # Parameters
    ///
    /// * `metadata`: A `ManagerMetadata` struct containing metadata information needed for
    ///   constructing the Manager instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the constructed Manager instance or an error
    /// message wrapped in a `Box<dyn Error>`. If the Manager is successfully constructed, it
    /// returns `Ok(Manager)`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of constructing the
    /// Manager, such as failure to load the configuration file, invalid configuration
    /// parameters, or errors encountered while retrieving session information.
    pub fn new(metadata: ManagerMetadata) -> Result<Self, Box<dyn Error>> {
        // Check if the configuration file exists
        if Path::new(metadata.config_path.as_str()).exists() {
            // Load the configuration file
            let config = Ini::load_from_file(metadata.config_path.clone())?;
            // Check for the autologin section in the configuration
            if let Some(autologin_section) = config.section(Some(metadata.autologin_section_name.clone())) {
                let autologin_session = autologin_section.get(metadata.autologin_session_key_name.clone());
                let autologin_user = autologin_section.get(metadata.autologin_user_key_name.clone());
                // Initialize the Manager instance with autologin information if available
                return Ok(Self {
                    autologin: if let Some(autologin_session) = autologin_session {
                        // Determine if autologin is enabled based on the session
                        autologin_session == &format!("{MOLYUU_REDIRECT_SESSION_PREFIX}-wayland") ||
                            autologin_session == &format!("{MOLYUU_REDIRECT_SESSION_PREFIX}-x11")
                    } else {
                        false
                    },
                    session_type: {
                        let oneshot_session = Session::get_oneshot_session()?;
                        // Determine the session type based on the availability of one-shot session
                        if let Some(oneshot_session) = oneshot_session {
                            oneshot_session.get_protocol()
                        } else {
                            Session::get_default_session()?.get_protocol()
                        }
                    },
                    login_user: autologin_user.map(|user| String::from(user)),
                    metadata: metadata.clone(),
                });
            }
        }

        // Initialize the Manager instance with default values
        Ok(Self {
            autologin: false,
            session_type: {
                let oneshot_session = Session::get_oneshot_session()?;
                // Determine the session protocol
                if let Some(oneshot_session) = oneshot_session {
                    oneshot_session.get_protocol()
                } else {
                    Session::get_default_session()?.get_protocol()
                }
            },
            login_user: None,
            metadata: metadata.clone(),
        })
    }

    pub fn set_auto_login(&mut self, enabled: bool) -> Result<(), Box<dyn Error>> {
        self.autologin = enabled;
        self.save_config()?;
        Ok(())
    }

    pub fn set_login_user(&mut self, user: &str) -> Result<(), Box<dyn Error>> {
        self.login_user = Some(String::from(user));
        self.save_config()?;
        Ok(())
    }

    pub fn set_as_default_manager(&self) -> Result<(), Box<dyn Error>> {
        let login_info = GLOBAL_CONFIG.get_mut().unwrap().get("login").as_table_mut().unwrap();
        login_info["manager"] = Value::String(String::from(self.metadata.systemd_unit.as_str()));
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    pub fn login_now(&self) -> Result<(), Box<dyn Error>> {
        self.save_config()?;
        SYSTEMCTL.lock().unwrap().reset_failed_unit(self.metadata.systemd_unit.as_str())?;
        SYSTEMCTL.lock().unwrap().restart_unit(self.metadata.systemd_unit.as_str())?;
        Ok(())
    }

    pub fn update_metadata(&mut self, metadata: ManagerMetadata) -> Result<(), Box<dyn Error>> {
        self.metadata = metadata;
        Ok(())
    }

    /// Save the configuration
    ///
    /// This function updates or creates the configuration file with the current settings. It manages
    /// the configuration for autologin and login user, ensuring that the autologin session and user
    /// are correctly set based on the current state. After updating the configuration file, it also
    /// updates the program global configuration accordingly.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of saving the configuration. If the
    /// configuration is successfully saved, it returns `Ok(())`. If an error occurs during the
    /// process, it returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of saving the
    /// configuration, such as failure to load or create the configuration file, inability to write
    /// to the file, or errors encountered while updating global configuration.
    pub fn save_config(&self) -> Result<(), Box<dyn Error>> {
        // Load existing configuration or create a new one
        let mut config = if Path::new(self.metadata.config_path.as_str()).exists() {
            Ini::load_from_file(self.metadata.config_path.as_str())?
        } else {
            if !Path::new(self.metadata.config_path.as_str()).parent().unwrap().exists() {
                std::fs::create_dir_all(Path::new(self.metadata.config_path.as_str()).parent().unwrap())?;
            }
            Ini::new()
        };

        // Configure autologin session based on the current state
        let mut autologin_section = config.with_section(Some(self.metadata.autologin_section_name.as_str()));
        if self.autologin {
            match self.session_type {
                Protocol::X11 => {
                    autologin_section.set(self.metadata.autologin_session_key_name.as_str(), format!("{MOLYUU_REDIRECT_SESSION_PREFIX}-x11"));
                }
                Protocol::Wayland => {
                    autologin_section.set(self.metadata.autologin_session_key_name.as_str(), format!("{MOLYUU_REDIRECT_SESSION_PREFIX}-wayland"));
                }
            }
        } else {
            autologin_section.delete(&self.metadata.autologin_session_key_name.as_str());
        }

        // Set login user if provided
        if let Some(login_user) = &self.login_user {
            let mut autologin_section = config.with_section(Some(self.metadata.autologin_section_name.as_str()));
            autologin_section.set(self.metadata.autologin_user_key_name.as_str(), login_user.as_str());
        }

        // Write configuration to a buffer and then to the file
        let mut buffer = BufWriter::new(Vec::new());
        config.write_to(&mut buffer)?;
        privilege::write(String::from_utf8_lossy(&*buffer.into_inner()?).as_ref(), self.metadata.config_path.as_str())?;

        // Update program global configuration
        self.update_global_config()?;

        Ok(())
    }

    pub fn update_global_config(&self) -> Result<(), Box<dyn Error>> {
        let login_info = GLOBAL_CONFIG.get_mut().unwrap().get("login").as_table_mut().unwrap();
        let autologin_info = login_info.get_mut("autologin").unwrap().as_table_mut().unwrap();
        autologin_info["enable"] = Value::Boolean(self.autologin);
        if self.login_user.is_some() {
            autologin_info["user"] = Value::String(self.login_user.clone().unwrap());
        }
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }
}

pub fn get_current_manager() -> Result<Manager, Box<dyn Error>> {
    let login_info = GLOBAL_CONFIG.get_mut().unwrap().get("login").as_table().unwrap();
    let current_manager = login_info.get("manager");
    if current_manager.is_some() {
        let manager_name = String::from(current_manager.unwrap().as_str().unwrap());
        match manager_name.as_str() {
            "lightdm" => {
                return Ok(ManagerBuilder::new().use_manager(SupportedManager::LightDM).build()?);
            }
            "sddm" => {
                return Ok(ManagerBuilder::new().use_manager(SupportedManager::SDDM).build()?);
            }
            _ => {}
        }
    }
    Err(Box::from("Unsupported manager"))
}

pub fn set_manager(new_manager: &str) -> Result<(), Box<dyn Error>> {
    let login_info = GLOBAL_CONFIG.get_mut().unwrap().get("login").as_table_mut().unwrap();
    let current_manager = login_info.get("manager");
    if current_manager.is_some() {
        let manager = String::from(current_manager.unwrap().as_str().unwrap());

        if manager == new_manager.to_lowercase() {
            return Err(Box::from("Specific manager is already current login manager"));
        }

        match new_manager {
            "lightdm" => {
                let mut manager = get_current_manager()?;
                manager.update_metadata(ManagerMetadata::build_for_supported_manager(SupportedManager::LightDM))?;
                manager.save_config()?;
                manager.set_as_default_manager()?;
            }
            "sddm" => {
                let mut manager = get_current_manager()?;
                manager.update_metadata(ManagerMetadata::build_for_supported_manager(SupportedManager::SDDM))?;
                manager.save_config()?;
                manager.set_as_default_manager()?;
            }
            _ => {
                return Err(Box::from("Unsupportted manager"));
            }
        }
    } else {
        let manager = ManagerBuilder::new().use_manager(match new_manager {
            "lightdm" => SupportedManager::LightDM,
            "sddm" => SupportedManager::SDDM,
            _ => {
                return Err(Box::from("Unsupportted manager"));
            }
        }).build()?;
        manager.save_config()?;
        manager.set_as_default_manager()?;
    }
    Ok(())
}