use air::client::{Client, ClientConfig};
use air::host::{Custom, OpenAI};
use air::transcript::{load, Transcript};
use air::Message;
use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use inquire::{Password, Text};
use rustyline::{error::ReadlineError, DefaultEditor};
use serde::Serialize;
use std::convert::From;
use std::fs::File;
use std::io::{stdout, LineWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use url::Url;

mod profile;
use profile::Profile;

#[derive(clap::ValueEnum, Copy, Clone, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Host {
    #[default]
    Custom,
    OpenAI,
}

#[derive(Parser, Default, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
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
    input: Option<PathBuf>,

    #[clap(short, long, default_value_t = false)]
    /// Verbose output
    verbose: bool,

    #[clap(short, long, default_value = None)]
    profile: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Subcommand)]
enum ProfileCommands {
    /// Add a new profile
    Add { name: Option<String> },

    /// Remove an existing profile
    Remove { name: String },

    /// List all profiles
    List,
}

#[derive(Clone, clap::Args, Debug)]
struct ProfileArgs {
    /// Add a new profile
    #[command(subcommand)]
    command: ProfileCommands,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Manage profiles
    Profile(ProfileArgs),
}

impl From<Args> for ClientConfig {
    fn from(value: Args) -> Self {
        Self {
            max_tokens: value.max_tokens,
            verbose: value.verbose,
            ..Default::default()
        }
    }
}

/// Main REPL for interacting with model providers.
fn repl<T: Write>(
    mut client: Client,
    mut transcript: Transcript<T>,
    profile: Profile,
) -> Result<()> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("{} (air v{VERSION})", client);
    println!("Using profile {}", profile.name);

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

fn main() -> Result<()> {
    let args = Args::parse();

    // handle profile commands
    if let Some(Command::Profile(profile_args)) = args.command {
        match profile_args.command {
            ProfileCommands::Add { name } => {
                let profile = Profile {
                    name: name
                        .unwrap_or_else(|| Text::new("Enter profile name: ").prompt().unwrap()),
                    key: Password::new("Enter API key: ")
                        .with_display_mode(inquire::PasswordDisplayMode::Masked)
                        .without_confirmation()
                        .prompt()?,
                };
                profile.save().expect("Failed to save profile");
                println!("Created profile {}", profile.name);
                return Ok(());
            }
            ProfileCommands::Remove { name } => {
                let profile = Profile::load(name.clone())?;
                profile.delete().expect("Failed to remove profile");
                println!("Removed profile {}", name);
                return Ok(());
            }
            ProfileCommands::List => {
                let profiles = Profile::list().expect("Failed to list profiles");
                for profile in profiles {
                    println!("{}", profile.name);
                }
                return Ok(());
            }
        }
    }

    // otherwise load profile from args or environment
    let profile = match args.profile {
        None => {
            if dotenv().is_ok() {
                println!("Loaded .env file");
            };
            let key = std::env::var("API_KEY").expect(
                "No credentials found. You must select an existing profile or set the environment variable `API_KEY`",
                );
            Profile {
                name: "from environment".to_string(),
                key,
            }
        }
        Some(ref name) => Profile::load(name.clone())?,
    };

    let context = match args.input {
        None => Vec::new(),
        Some(ref path) => {
            let file = File::open(path)?;
            load(file)?
        }
    };

    // create client based on args, key, context, etc.
    let client = match args.host {
        Host::OpenAI => {
            let name = args.name.clone().unwrap_or("gpt-3.5-turbo".to_string());
            let provider = OpenAI::new(name, profile.key.clone());
            Client::new(provider)
        }
        Host::Custom => {
            let provider = Custom::new(Url::from_str("localhost:8000")?);
            Client::new(provider)
        }
    }
    .with(args.clone().into())
    .with_context(context);

    // setup transcript to record on calls to `record` if output is provided
    let mut writer: Option<LineWriter<_>> = args
        .output
        .map(File::create)
        .transpose()?
        .map(LineWriter::new);
    let transcript = Transcript::conditionally(writer.as_mut());

    repl(client, transcript, profile)
}
