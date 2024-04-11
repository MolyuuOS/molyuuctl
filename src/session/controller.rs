use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use std::string::String;

use ini::Ini;
use toml::{Table, Value};
use crate::config::GLOBAL_CONFIG;

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

    /// Generate a Session instance based on the configuration specified in a file.
    ///
    /// # Parameters
    ///
    /// * `session_name`: An optional parameter representing the name of the session to be retrieved
    ///   from the configuration file. If provided, the session with the corresponding name will be
    ///   retrieved. If not provided (i.e., `None`), the default session will be used.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the generated `Session` instance or an error message wrapped in
    /// a `Box<dyn Error>`. If the session is successfully created, it returns `Ok(Session)`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of generating the
    /// session, such as failure to read the configuration file or invalid configuration parameters.
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

    /// Search session in the system.
    ///
    /// This function searches for the specified session in the system by looking in the following directories:
    /// 1. /usr/share/xsessions
    /// 2. /usr/share/wayland-sessions.
    ///
    /// # Parameters
    /// - `real_session_name`: The real session name of the session to search for.
    ///
    /// # Returns
    /// A Result containing the Protocol of the session if found, or an error if the session is not found or
    /// if there is an issue with accessing the system directories.
    ///
    /// # Errors
    /// Returns an Error if session is not found in searching paths.
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

    /// Start the session as specified by the desktop file, executing the appropriate command.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of starting the session. If the session
    /// is successfully started, it returns `Ok(())`. If an error occurs during the process, it
    /// returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of starting the session,
    /// such as failure to load the session configuration file, inability to retrieve necessary
    /// information from the desktop file, or failure to execute the session command.
    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        // Determine the location of the session desktop file based on the protocol
        let session_file_location = if self.protocol == Protocol::X11 {
            SYSTEM_XSESSIONS_PATH
        } else if self.protocol == Protocol::Wayland {
            SYSTEM_WAYLAND_SESSIONS_PATH
        } else {
            return Err(Box::from("Unknown protocol"));
        };

        // Load the session desktop file
        let session_real_name = self.real_name.as_str();
        let session_file = Ini::load_from_file(format!("{session_file_location}/{session_real_name}.desktop"))?;

        // Extract the necessary information from the desktop file
        let desktop_section = session_file.section(Some("Desktop Entry")).unwrap();
        let command = desktop_section.get("Exec").unwrap();
        println!("Target Session: {}", desktop_section.get("Name").unwrap());
        println!("Executing Session Command: {}", command);

        // Execute the session command
        Command::new("/bin/bash")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to launch session");

        Ok(())
    }


    /// Start either a one-shot session or the default session as specified in the global configuration.
    ///
    /// This function retrieves session information from the global configuration, checks if a
    /// one-shot session is configured and not already started. If so, it starts the specified
    /// one-shot session; otherwise, it starts the default session. After starting the session, it
    /// updates the login manager configuration accordingly.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of starting the session. If the session
    /// is successfully started, it returns `Ok(())`. If an error occurs during the process, it
    /// returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of starting the session,
    /// such as failure to retrieve session information from the global configuration, failure to
    /// update the configuration, or errors encountered while starting the session itself.
    pub fn start_oneshot_or_default_session() -> Result<(), Box<dyn Error>> {
        // Retrieve session information from the global configuration
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        let oneshot_session = session_info.get("oneshot_session");
        let oneshot_started = session_info.get("oneshot_started");

        // Check if a one-shot session is configured and not already started
        if let (Some(oneshot_session), Some(oneshot_started)) = (oneshot_session, oneshot_started) {
            if !oneshot_started.as_bool().unwrap() {
                let session_to_start = String::from(oneshot_session.as_str().unwrap());
                session_info["oneshot_started"] = Value::Boolean(true);
                GLOBAL_CONFIG.get_mut().unwrap().save_config();

                // Start the specified one-shot session
                Self::from_config(Some(session_to_start.as_str()))?.start()?;
            }
        } else {
            // If no one-shot session is configured or it's already started, start the default session
            Self::from_config(None)?.start()?;
        }

        // Update Login Manager config
        get_current_manager()?.save_config()?;
        Ok(())
    }

    /// Execute the logout command to end the current user session.
    ///
    /// This function executes the logout command, if set, to end the current user session. If no
    /// logout command is configured, it returns an error indicating that the logout command is not
    /// set, and the logout operation cannot be performed.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of the logout operation. If the logout
    /// operation is successfully executed, it returns `Ok(())`. If an error occurs during the process,
    /// it returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of executing the logout
    /// command, such as failure to retrieve the logout command or errors encountered while executing
    /// the command itself.
    pub fn logout(&self) -> Result<(), Box<dyn Error>> {
        // Check if a logout command is set
        if self.logout_command.is_none() {
            return Err(Box::from("No logout command is set, cannot logout"));
        }

        // Execute the logout command
        Command::new("/bin/bash")
            .arg("-c")
            .arg(self.logout_command.as_ref().unwrap().as_str())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to logout session");

        Ok(())
    }

    /// Rename the session with a new name.
    ///
    /// This function renames the session by updating its registered name in the global configuration.
    /// It first checks if a session with the new name already exists; if so, it returns an error.
    /// Otherwise, it updates the session's name in the configuration, updates the default session
    /// if necessary, and saves the configuration.
    ///
    /// # Parameters
    ///
    /// * `new_name`: The new name to assign to the session.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of renaming the session. If the session
    /// is successfully renamed, it returns `Ok(())`. If an error occurs during the process, it
    /// returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if the new name conflicts with an existing session name or if there are
    /// issues encountered during the process of renaming the session or saving the configuration.
    pub fn rename(&mut self, new_name: &str) -> Result<(), Box<dyn Error>> {
        // Retrieve session information from the global configuration
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();

        // Check if a session with the new name already exists
        if session_info.get(new_name).is_some() {
            return Err(Box::from("Specific session already exists"));
        }

        // Store the current name of the session
        let old_name = self.reg_name.clone();

        // Retrieve information about the current session
        let current_session_info = session_info.get(self.reg_name.as_str()).unwrap();

        // Update session name in the configuration
        session_info.insert(String::from(new_name), current_session_info.clone());
        session_info.remove(&self.reg_name);
        self.reg_name = String::from(new_name);

        // Update default session if necessary
        if session_info.get("default").unwrap().as_str() == Some(old_name.as_str()) {
            session_info["default"] = Value::String(self.reg_name.clone());
        }

        // Save the updated configuration
        GLOBAL_CONFIG.get_mut().unwrap().save_config();

        Ok(())
    }

    /// Remove the session configuration from the global configuration.
    ///
    /// This function removes the session configuration identified by its registered name from the
    /// global configuration. It first checks if the session to be removed is the default session;
    /// if so, it returns an error indicating that the default session cannot be removed. Otherwise,
    /// it removes the session from the configuration, saves the updated configuration, and returns
    /// successfully.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of removing the session configuration.
    /// If the session configuration is successfully removed, it returns `Ok(())`. If an error occurs
    /// during the process, it returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of removing the session
    /// configuration, such as attempting to remove the default session or failure to save the updated
    /// configuration.
    pub fn remove(&self) -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        if session_info.get("default").unwrap().as_str() == Some(self.reg_name.as_str()) {
            return Err(Box::from("Cannot remove default session"));
        }
        session_info.remove(&self.reg_name);
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    /// Register a new session configuration in the global configuration.
    ///
    /// This function registers a new session configuration in the global configuration. It first
    /// checks if a session with the same registered name already exists; if so, it returns an error.
    /// Otherwise, it constructs a new session table with the necessary information such as the
    /// session name, protocol, and logout command (if provided). It then inserts this new session
    /// into the session information in the global configuration, saves the updated configuration,
    /// and returns successfully.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of registering the session configuration.
    /// If the session configuration is successfully registered, it returns `Ok(())`. If an error occurs
    /// during the process, it returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of registering the session
    /// configuration, such as attempting to register a session with a duplicate name or an unknown
    /// protocol, or failure to save the updated configuration.
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
        if let Some(logout_command) = &self.logout_command {
            new_table.insert(String::from("logoutcmd"), Value::String(logout_command.clone()));
        }
        session_info.insert(String::from(&self.reg_name), Value::Table(new_table));
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    /// Set the logout command for the current session.
    ///
    /// # Parameters
    ///
    /// * `command`: A string representing the command to be executed when logging out of the session.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of setting the logout command. If the
    /// command is successfully set, it returns `Ok(())`. If an error occurs during the process, it
    /// returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of setting the logout
    /// command, such as failure to access or modify the global configuration or errors encountered
    /// while saving the configuration.
    pub fn set_logout_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        let current_session_section = session_info.get_mut(self.reg_name.as_str()).unwrap().as_table_mut().unwrap();
        current_session_section.insert(String::from("logoutcmd"), toml::Value::String(String::from(command)));
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    /// Set the current session as the default session in the global configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of setting the session as default. If the
    /// session is successfully set as default, it returns `Ok(())`. If an error occurs during the
    /// process, it returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of setting the session as
    /// default, such as failure to access or modify the global configuration or errors encountered
    /// while saving the configuration.
    pub fn set_as_default(&self) -> Result<(), Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table_mut().unwrap();
        session_info["default"] = toml::Value::String(self.reg_name.clone());
        GLOBAL_CONFIG.get_mut().unwrap().save_config();
        Ok(())
    }

    /// Set the current session as a one-shot session in the global configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the success or failure of setting the session as a one-shot
    /// session. If the session is successfully set as a one-shot session, it returns `Ok(())`. If an
    /// error occurs during the process, it returns an error message wrapped in a `Box<dyn Error>`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of setting the session as
    /// a one-shot session, such as failure to access or modify the global configuration, errors
    /// encountered while saving the configuration, or errors while updating the login manager
    /// configuration for session changes.
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

        // Update Login Manager config to reflect the session change
        get_current_manager()?.save_config()?;
        Ok(())
    }

    /// Retrieve the protocol associated with the session.
    ///
    /// # Returns
    ///
    /// Returns the protocol (`Protocol`) associated with the session.
    pub fn get_protocol(&self) -> Protocol {
        self.protocol
    }

    /// Retrieve the default session configuration.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the default session configuration (`Self`) or an error
    /// message wrapped in a `Box<dyn Error>`. If the default session configuration is successfully
    /// retrieved, it returns `Ok(Self)`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of retrieving the default
    /// session configuration, such as failure to load the configuration from the file.
    pub fn get_default_session() -> Result<Self, Box<dyn Error>> {
        Ok(Self::from_config(None)?)
    }

    /// Retrieve the one-shot session configuration if it exists and is not already started.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either an optional one-shot session configuration (`Option<Self>`)
    /// or an error message wrapped in a `Box<dyn Error>`. If a one-shot session configuration exists
    /// and is not already started, it returns `Ok(Some(Self))`. If there is no one-shot session
    /// configuration or it's already started, it returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are issues encountered during the process of retrieving the
    /// one-shot session configuration, such as failure to load the configuration from the file.
    pub fn get_oneshot_session() -> Result<Option<Self>, Box<dyn Error>> {
        let session_info = GLOBAL_CONFIG.get_mut().unwrap().get("session").as_table().unwrap();
        let oneshot_session = session_info.get("oneshot_session");
        let oneshot_started = session_info.get("oneshot_started");

        if let (Some(oneshot_session), Some(oneshot_started)) = (oneshot_session, oneshot_started) {
            if !oneshot_started.as_bool().unwrap() {
                return Ok(Some(Self::from_config(Some(oneshot_session.as_str().unwrap()))?));
            }
        }
        Ok(None)
    }
}