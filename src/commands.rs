use chrono::Utc;
use feed_rs::parser;
use futures::{future::join_all, Future};
use std::{collections::HashMap, error::Error, process::Command};

pub trait BotCommand {
    fn execute(&self) -> impl Future<Output = Result<String, Box<dyn Error>>>;
    fn usage(&self) -> String;
}

pub struct DateCommand;

impl BotCommand for DateCommand {
    async fn execute(&self) -> Result<String, Box<dyn Error>> {
        Ok(Utc::now().to_rfc2822())
    }

    fn usage(&self) -> String {
        String::from("Show current time")
    }
}

pub struct FeedsCommand<'a> {
    pub urls: &'a Option<HashMap<String, String>>,
}

impl<'a> FeedsCommand<'a> {
    pub fn new(urls: &'a Option<HashMap<String, String>>) -> Self {
        Self { urls }
    }
}

impl<'a> BotCommand for FeedsCommand<'a> {
    async fn execute(&self) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        let feed_urls = self.urls.as_ref().ok_or("Could not find feed URLs")?;
        let mut futures = Vec::new();

        for (_, url) in feed_urls.iter() {
            let client = reqwest::Client::new();
            futures.push(client.get(url).header("User-Agent", "seribot").send());
        }

        let results = join_all(futures).await;

        for result in results {
            let feed_text = result?.text().await?;
            let feed = parser::parse(feed_text.as_bytes())?;
            let entries = feed
                .entries
                .iter()
                .map(|e| {
                    let title = match e.title.to_owned() {
                        Some(title) => title.content,
                        None => String::from("N/A"),
                    };
                    let summary = match e.summary.clone() {
                        Some(summary) => summary.content,
                        None => String::from("N/A"),
                    };

                    format!("{}\r\n\r\n{}", title, summary)
                })
                .collect::<Vec<String>>()
                .join("\r\n\r\n-----\r\n\r\n");

            output = format!("{}\r\n{}\r\n", output, entries);
        }

        Ok(output)
    }

    fn usage(&self) -> String {
        String::from("Show RSS/Atom feeds")
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
    async fn execute(&self) -> Result<String, Box<dyn Error>> {
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
