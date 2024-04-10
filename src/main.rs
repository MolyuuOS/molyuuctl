use std::string::String;

use clap::{arg, Command};

use crate::login::manager::get_current_manager;
use crate::session::controller::Session;
use crate::session::protocol::Protocol;
use crate::tools::cleanup::cleanup;

mod config;
mod session;
mod login;
mod tools;

fn cli() -> Command {
    Command::new("MolyuuOS System Controller")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("session")
            .about("Sessions settings")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(Command::new("register")
                .about("Register a new session")
                .arg_required_else_help(true)
                .arg(arg!(-n --name <REGISTER_NAME> "Register Name")
                    .required(true))
                .arg(arg!(-s --session <SESSION_NAME> "Session name")
                    .required(true))
                .arg(arg!(-p --protocol <PROTOCOL_TYPE> "Protocol")
                    .value_parser(["auto", "wayland", "x11"])
                    .default_value("auto")
                    .default_missing_value("auto"))
                .arg(arg!(-l --logout <LOGOUT_COMMAND> "Session logout command")))
            .subcommand(Command::new("set-default")
                .about("Set default session")
                .arg_required_else_help(true)
                .arg(arg!([register_name] "Register name")
                    .required(true)))
            .subcommand(Command::new("set-logout-command")
                .about("Set logout command for specific session")
                .arg_required_else_help(true)
                .arg(arg!([register_name] "Register name")
                    .required(true))
                .arg(arg!([logout_command] "Logout commnad")
                    .required(true)))
            .subcommand(Command::new("rename")
                .about("Rename a session")
                .arg_required_else_help(true)
                .arg(arg!([original_name] "Old register name of the session")
                    .required(true))
                .arg(arg!([new_name] "New register name of the session")
                    .required(true)))
            .subcommand(Command::new("remove")
                .about("Remove a registered session")
                .arg_required_else_help(true)
                .arg(arg!([register_name] "Session register name")
                    .required(true)))
            .subcommand(Command::new("start")
                .about("Start a session")
                .arg(arg!([register_name] "Session register name")
                    .default_value("default")
                    .default_missing_value("default")))
            .subcommand(Command::new("logout")
                .about("Logout specific session")
                .arg_required_else_help(true)
                .arg(arg!([register_name] "Session register name")
                    .required(true)))
            .subcommand(Command::new("set-oneshot")
                .about("Set a session to start oneshot while login with set login manager next time")
                .arg_required_else_help(true)
                .arg(arg!([register_name] "Session register name")
                    .required(true))))
        .subcommand(Command::new("login")
            .about("Login settings")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(Command::new("set-manager")
                .about("Set Login Manager (Currently supported: lightdm, sddm)")
                .arg_required_else_help(true)
                .arg(arg!([manager_name] "Login Manager Name")
                    .required(true)
                    .value_parser(["lightdm", "sddm"])))
            .subcommand(Command::new("autologin")
                .about("Config Auto Login")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(Command::new("enable")
                    .about("Enable Auto Login")
                    .arg(arg!(-u --user <USERNAME> "User that login as")))
                .subcommand(Command::new("disable")
                    .about("Disable Auto Login")))
            .subcommand(Command::new("now")
                .about("Login via set Login Manager now")))
}

fn main() {
    unsafe {
        libc::signal(libc::SIGINT, cleanup as libc::sighandler_t);
        libc::signal(libc::SIGTERM, cleanup as libc::sighandler_t);
    }

    let matches = cli().get_matches();
    config::helper::Configuration::init(None);

    match matches.subcommand() {
        Some(("session", sub_m)) => {
            match sub_m.subcommand() {
                Some(("register", session_sub_m)) => {
                    let reg_name = session_sub_m.get_one::<String>("name").expect("required");
                    let session_name = session_sub_m.get_one::<String>("session").expect("required");
                    let protocol_str = session_sub_m.get_one::<String>("protocol").expect("required");
                    let logout_command = session_sub_m.get_one::<String>("logout");
                    let protocol = {
                        let protocol_str_lower = protocol_str.to_lowercase();
                        match protocol_str_lower.as_str() {
                            "x11" => {
                                Some(Protocol::X11)
                            }
                            "wayland" => {
                                Some(Protocol::Wayland)
                            }
                            "auto" => None,
                            _ => panic!("Unknown protocol")
                        }
                    };
                    Session::new(reg_name.clone(), session_name.clone(), logout_command.cloned(), protocol).unwrap().register().unwrap()
                }
                Some(("set-default", session_sub_m)) => {
                    let register_name = session_sub_m.get_one::<String>("register_name").expect("required");
                    Session::from_config(Some(register_name.as_str())).unwrap().set_as_default().unwrap()
                }
                Some(("set-oneshot", session_sub_m)) => {
                    let register_name = session_sub_m.get_one::<String>("register_name").expect("required");
                    Session::from_config(Some(register_name.as_str())).unwrap().set_start_oneshot().unwrap();
                }
                Some(("set-logout-command", session_sub_m)) => {
                    let register_name = session_sub_m.get_one::<String>("register_name").expect("required");
                    let logout_command = session_sub_m.get_one::<String>("logout_command").expect("required");
                    Session::from_config(Some(register_name.as_str())).unwrap().set_logout_command(logout_command.as_str()).unwrap()
                }
                Some(("rename", session_sub_m)) => {
                    let original_name = session_sub_m.get_one::<String>("original_name").expect("required");
                    let new_name = session_sub_m.get_one::<String>("new_name").expect("required");
                    Session::from_config(Some(original_name.as_str())).unwrap().rename(new_name.as_str()).unwrap()
                }
                Some(("remove", session_sub_m)) => {
                    let register_name = session_sub_m.get_one::<String>("register_name").expect("required");
                    Session::from_config(Some(register_name.as_str())).unwrap().remove().unwrap()
                }
                Some(("start", session_sub_m)) => {
                    let register_name = session_sub_m.get_one::<String>("register_name").expect("required");
                    if register_name.as_str() == "default" {
                        Session::start_oneshot_or_default_session().unwrap()
                    } else {
                        Session::from_config(Some(register_name.as_str())).unwrap().start().unwrap()
                    }
                }
                Some(("logout", session_sub_m)) => {
                    let register_name = session_sub_m.get_one::<String>("register_name").expect("required");
                    Session::from_config(Some(register_name.as_str())).unwrap().logout().unwrap()
                }
                _ => {}
            }
        }
        Some(("login", sub_m)) => {
            match sub_m.subcommand() {
                Some(("set-manager", login_sub_m)) => {
                    let manager_name = login_sub_m.get_one::<String>("manager_name").expect("required");
                    login::manager::set_manager(manager_name.to_lowercase().as_str()).unwrap();
                }
                Some(("autologin", login_sub_m)) => {
                    match login_sub_m.subcommand() {
                        Some(("enable", autologin_enable_sub_m)) => {
                            let username = autologin_enable_sub_m.get_one::<String>("user");
                            if username.is_some() {
                                get_current_manager().unwrap().set_login_user(username.unwrap().as_str()).unwrap();
                            }
                            get_current_manager().unwrap().set_auto_login(true).unwrap();
                        }
                        Some(("disable", _)) => {
                            get_current_manager().unwrap().set_auto_login(false).unwrap();
                        }
                        _ => {}
                    }
                }
                Some(("now", _)) => get_current_manager().unwrap().login_now().unwrap(),
                _ => {}
            }
        }
        _ => {}
    }
}
