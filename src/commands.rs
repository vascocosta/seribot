use chrono::Utc;
use std::{error::Error, process::Command};

pub trait BotCommand {
    fn execute(&self) -> Result<String, Box<dyn Error>>;
    fn usage(&self) -> String;
}

pub struct DateCommand;

impl BotCommand for DateCommand {
    fn execute(&self) -> Result<String, Box<dyn Error>> {
        Ok(Utc::now().to_rfc2822())
    }

    fn usage(&self) -> String {
        String::from("Show current time")
    }
}

pub struct ShellCommand {
    name: String,
    args: Option<Vec<String>>,
}

impl ShellCommand {
    pub fn new(name: &str, args: Option<Vec<String>>) -> Self {
        Self {
            name: name.to_owned(),
            args,
        }
    }
}

impl BotCommand for ShellCommand {
    fn execute(&self) -> Result<String, Box<dyn Error>> {
        match Command::new(self.name.clone())
            .args(self.args.clone().unwrap_or_default())
            .output()
        {
            Ok(output) => Ok(String::from_utf8_lossy(&output.stdout).replace('\n', "\r\n")),
            Err(_) => Ok(String::from("Could not run command")),
        }
    }

    fn usage(&self) -> String {
        String::from("Run a shell command")
    }
}
