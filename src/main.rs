use air::client::Client;
use air::host::{Custom, OpenAI};
use air::transcript::{load, Transcript};
use air::Message;
use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use rustyline::{error::ReadlineError, DefaultEditor};
use serde::Serialize;
use std::fs::File;
use std::io::{stdout, LineWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use url::Url;

#[derive(clap::ValueEnum, Clone, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Host {
    #[default]
    Custom,
    OpenAI,
}

#[derive(Parser, Debug)]
struct Args {
    /// Host for the model
    #[clap(long, value_enum, default_value_t = Host::OpenAI)]
    host: Host,

    /// Name of host model, if applicable
    #[clap(short, long, default_value = None)]
    name: Option<String>,

    /// Maximum context size in tokens to allow; useful for billing purposes
    #[clap(short, long, default_value = None)]
    max_tokens: Option<usize>,

    #[clap(short, long, default_value = None)]
    /// Output location to save transcript
    output: Option<PathBuf>,

    #[clap(short, long, default_value = None)]
    /// Location to load transcript for context initialization
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    dotenv()?;
    let args = Args::parse();
    let context = match args.file {
        None => Vec::new(),
        Some(path) => {
            let file = File::open(path)?;
            load(file)?
        }
    };

    let mut client = match args.host {
        Host::OpenAI => {
            let key = std::env::var("API_KEY")
                .expect("You must set the environment variable `API_KEY` to your OpenAI API key");
            let name = args.name.clone().unwrap_or("gpt-3.5-turbo".to_string());
            let provider = OpenAI::new(name, key);
            Client::new(provider).with_context(context)
        }
        Host::Custom => {
            let provider = Custom::new(Url::from_str("localhost:8000")?);
            Client::new(provider).with_context(context)
        }
    };

    // header
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!(
        "{} {} (air v{VERSION})",
        client,
        &args.name.clone().unwrap_or("".to_string())
    );

    // setup transcript to record on calls to `record` if output is provided
    let mut writer: Option<LineWriter<_>> = args
        .output
        .map(File::create)
        .transpose()?
        .map(LineWriter::new);
    let mut transcript = Transcript::conditionally(writer.as_mut())?;

    // REPL
    let mut response: &Message;
    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("exit");
                break;
            }
            Err(err) => {
                println!("error: {:?}", err);
                break;
            }
            Ok(line) if line.is_empty() => continue,
            Ok(line) => {
                let message = Message::user(line);
                transcript.record(&message)?;
                response = match client.send(message) {
                    Ok(response) => response,
                    Err(err) => {
                        eprintln!("error: {}", err);
                        continue;
                    }
                };
                transcript.record(response)?;

                // ChatGPT-style rolling output with newline formatting
                let mut stdout = stdout().lock();
                for line in response.content.lines() {
                    for word in line.split_whitespace() {
                        write!(stdout, "{} ", word)?;
                        stdout.flush()?;
                        sleep(Duration::from_millis(100));
                    }
                }
                writeln!(stdout)?;
            }
        }
    }

    Ok(())
}
