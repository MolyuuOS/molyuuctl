use crate::config::helper::GLOBAL_CONFIG;
use crate::login::manager::get_current_manager;

pub extern "C" fn cleanup(sig: libc::c_int) {
    println!("Received SIGNAL: {}", sig);
    println!("Clean up before exit ...");

    // Save all configs
    GLOBAL_CONFIG.get_mut().unwrap().save_config();
    get_current_manager().unwrap().save_config().unwrap();

    println!("Done! Goodbye!");
}