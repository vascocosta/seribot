use serde::{Deserialize, Serialize};
use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::io::Write;
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    path::Path,
};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub serial: Serial,
    pub feeds: Option<HashMap<String, String>>,
}

impl Config {
    pub fn read<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        Ok(toml::from_str(
            &fs::read_to_string(path).map_err(|_| "Could not read config")?,
        )?)
    }

    pub fn _write<P>(&self, path: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        Ok(write!(
            File::create(path)?,
            "{}",
            toml::to_string(self).map_err(|_| "Syntax error in config")?
        )
        .map_err(|_| "Could not write to config")?)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "DataBits")]
enum DataBitsDef {
    Five,
    Six,
    Seven,
    Eight,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "FlowControl")]
pub enum FlowControlDef {
    None,
    Software,
    Hardware,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Parity")]
pub enum ParityDef {
    None,
    Odd,
    Even,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "StopBits")]
pub enum StopBitsDef {
    One,
    Two,
}

#[derive(Deserialize, Serialize)]
pub struct Serial {
    pub baud: u32,
    #[serde(with = "DataBitsDef")]
    pub data_bits: DataBits,
    #[serde(with = "FlowControlDef")]
    pub flow_control: FlowControl,
    #[serde(with = "ParityDef")]
    pub parity: Parity,
    pub port: String,
    #[serde(with = "StopBitsDef")]
    pub stop_bits: StopBits,
    pub timeout: u64,
}
