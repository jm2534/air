use ai_cli::client::Client;
use ai_cli::host::{Custom, OpenAI};
use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

use rustyline::{error::ReadlineError, DefaultEditor};
use serde::Serialize;

#[derive(clap::ValueEnum, Clone, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Host {
    #[default]
    Custom,
    OpenAI,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, value_enum, default_value_t = Host::OpenAI)]
    /// Host for the model
    host: Host,

    #[clap(short, long, default_value = None)]
    /// Name of host model, if applicable
    name: Option<String>,

    #[clap(short, long, default_value = None)]
    /// Maximum context size in tokens to allow; useful for billing purposes
    max_tokens: Option<usize>,
}

fn main() -> Result<()> {
    dotenv()?;
    let args = Args::parse();
    let mut client = match args.host {
        Host::OpenAI => {
            let key = std::env::var("API_KEY")?;
            let name = args.name.unwrap_or("gpt-3.5-turbo".to_string());
            let provider = OpenAI::new(name, key);
            Client::new(provider)
        }
        Host::Custom => {
            let provider = Custom::new("localhost:8000".into());
            Client::new(provider)
        }
    };

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("ai-cli v{VERSION}");

    let mut rl = DefaultEditor::new()?;
    let mut response: &str;

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) if line.is_empty() => continue,
            Ok(line) => {
                response = client.send(line)?;
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }

        let mut stdout = stdout().lock();
        for word in response.split_whitespace() {
            write!(stdout, "{} ", word)?;
            stdout.flush()?;
            sleep(Duration::from_millis(100));
        }
        writeln!(stdout)?;
    }

    Ok(())
}
