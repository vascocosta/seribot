mod config;

use chrono::Utc;
use config::Config;
use ihex::Record;
use std::{
    error::Error,
    io::{self, BufRead},
    process::Command,
    thread,
    time::Duration,
};

trait BotCommand {
    fn execute(&self) -> Result<String, Box<dyn Error>>;
    fn usage(&self) -> String;
}

struct DateCommand;

impl BotCommand for DateCommand {
    fn execute(&self) -> Result<String, Box<dyn Error>> {
        Ok(Utc::now().to_rfc2822())
    }

    fn usage(&self) -> String {
        String::from("Show current time")
    }
}

struct ShellCommand {
    name: String,
    args: Option<Vec<String>>,
}

impl ShellCommand {
    fn new(name: &str, args: Option<Vec<String>>) -> Self {
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

fn parse_command(line: String) -> Result<Box<dyn BotCommand>, Box<dyn Error>> {
    let mut parts = line.split_whitespace();
    let name = parts.next().ok_or("No command name")?.to_owned();
    let args: Vec<String> = parts.map(|a| a.to_owned()).collect();
    let args = if args.is_empty() { None } else { Some(args) };

    match name.as_str() {
        "date" => Ok(Box::new(DateCommand {})),
        "feeds" => todo!(),
        _ => Ok(Box::new(ShellCommand::new(&name, args))),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::read("config.toml")?;
    let port = serialport::new(&config.serial.port, config.serial.baud)
        .data_bits(config.serial.data_bits)
        .parity(config.serial.parity)
        .stop_bits(config.serial.stop_bits)
        .flow_control(config.serial.flow_control)
        .timeout(Duration::from_millis(config.serial.timeout))
        .open();

    match port {
        Ok(mut port) => {
            let mut reader = io::BufReader::new(port.try_clone()?);
            let mut line = String::new();

            loop {
                match reader.read_line(&mut line) {
                    Ok(_) => {
                        let command = parse_command(line.clone())?;
                        let output = format!("{}\r\n", command.execute()?);
                        let chunks = output.as_bytes().chunks(255);
                        let mut records: Vec<Record> = Vec::new();
                        let mut offset = 0;
                        let mut linear_address = 0;

                        for chunk in chunks {
                            if offset > 255 {
                                offset = 0;
                                linear_address += 5;
                                records.push(Record::ExtendedLinearAddress(linear_address));
                            }

                            records.push(Record::Data {
                                offset: offset * 255,
                                value: chunk.to_owned(),
                            });
                            offset += 1;
                        }

                        records.push(Record::EndOfFile);
                        thread::sleep(Duration::from_secs(1));

                        if let Ok(object) = ihex::create_object_file_representation(&records) {
                            port.write_all(object.as_bytes())?;
                        } else {
                            println!("Could not assemble Intel Hex object")
                        }

                        line.clear();
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => eprintln!("{:?}", e),
    }

    Ok(())
}
