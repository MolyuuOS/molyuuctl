use std::error::Error;
use std::process::{Command, Stdio};

pub fn write(value: &str, path: &str) -> Result<(), Box<dyn Error>> {
    Command::new("pkexec")
        .arg("/bin/bash")
        .arg("-c")
        .arg(format!("echo '{}' > '{}'", value, path))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;
    Ok(())
}