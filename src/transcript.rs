use crate::{Message, Role};
use anyhow::Result;
use enum_iterator::all;
use regex::Regex;
use std::{
    mem,
    io::{BufRead, Read, Write},
    str::FromStr,
};

fn role_regex() -> String {
    all::<Role>()
        .map(|r| r.to_string().to_uppercase() + ":")
        .collect::<Vec<String>>()
        .join("|")
}

/// Loads a transcript from a Read source (e.g. File, socket, etc.), returning
/// the contents as an ordered vector of `Message`s.
pub fn load(source: impl Read) -> Result<Vec<Message>> {
    let mut reader = std::io::BufReader::new(source);
    let mut messages = Vec::<Message>::new();

    // Initialize role or return early
    let mut role: Role;
    let mut buffer = String::with_capacity(1024);
    match reader.read_line(&mut buffer)? {
        0 => return Ok(messages),
        _ => role = Role::from_str(buffer.trim_end_matches(&[':', '\n'])).expect("Invalid start"),
    }
    buffer.clear();

    let re = Regex::new(&role_regex())?;
    while let Ok(n) = reader.read_line(&mut buffer) {
        if n == 0 {
            // EOF; save final message
            let message = Message::new(role, buffer.trim_end());
            messages.push(message);
            break;
        } else if let Some(r) = re.find_at(&buffer, buffer.len() - n) {
            // Found new role: save current message up to role and create new one
            let content = buffer[..buffer.len() - n].trim_end();
            let message = Message::new(role, content);
            messages.push(message);

            // Prepare next message's role and its buffer
            role = Role::from_str(r.as_str().trim_end_matches(&[':', '\n'])).unwrap();
            buffer.clear();
        }
        // Otherwise, keep appending to buffer
    }

    Ok(messages)
}

pub struct Transcript<'a, T: Write> {
    sink: Option<&'a mut T>,
}

impl<'a, T: Write> Transcript<'a, T> {
    pub fn new(sink: &'a mut T) -> Result<Self> {
        Ok(Self { sink: Some(sink) })
    }

    /// Create a new `Transcript` with an optional sink. If `None`, the sink
    /// is never written to.
    pub fn conditionally(sink: Option<&'a mut T>) -> Result<Self> {
        Ok(Self { sink })
    }

    /// Record a message to the transcript if a sink was provided on `Transcript`
    /// creation.
    pub fn record(&mut self, message: &Message) -> Result<()> {
        if let Some(&mut ref mut s) = self.sink{ 
            writeln!(s, "{}:", message.role.to_string().to_uppercase())?;
            for line in message.content.lines() {
                writeln!(s, "{line}")?;
            }
            writeln!(s)?;       
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Role;
    use enum_iterator::all;
    use std::{collections::HashSet, io::{Cursor, Seek}};

    /// Regex should match ROLE_0:|ROLE_1:|...ROLE_N: for an arbitrary ordering
    /// and definition of `Role`s.
    #[test]
    fn test_role_regex() {
        let re = role_regex();
        let re_set = HashSet::<String>::from_iter(re.split('|').map(|s| s.to_string()));
        let enum_set =
            HashSet::from_iter(all::<Role>().map(|r| r.to_string().to_uppercase() + ":"));
        assert!(re_set == enum_set, "{:?} != {:?}", re_set, enum_set);
    }

    #[test]
    fn test_transcript_single_format() -> Result<()> {
        let buffer = Vec::<u8>::new();
        let mut sink = Cursor::new(buffer);
        let mut transcript = Transcript::conditionally(Some(&mut sink))?;

        let message = Message {
            role: Role::User,
            content: "Hello, world!".to_string(),
        };
        transcript.record(&message)?;

        sink.rewind()?;
        let mut contents = String::new();
        sink.read_to_string(&mut contents)?;
        println!("{:?}", contents);

        assert_eq!(contents, "USER:\nHello, world!\n\n");
        Ok(())
    }

    #[test]
    fn test_transcript_interleaved() -> Result<()> {
        let buffer = Vec::<u8>::new();
        let mut sink = Cursor::new(buffer);
        let mut transcript = Transcript::conditionally(Some(&mut sink))?;

        let messages = vec![
            Message {
                role: Role::User,
                content: "Hello, assistant!".to_string(),
            },
            Message {
                role: Role::Assistant,
                content: "Hello, user!".to_string(),
            },
            Message {
                role: Role::Assistant,
                content: "Hello again, user!".to_string(),
            },
        ];
        for message in messages {
            transcript.record(&message)?;
        }

        sink.rewind()?;
        let mut contents = String::new();
        sink.read_to_string(&mut contents)?;
        assert_eq!(
            contents, 
            "USER:\nHello, assistant!\n\nASSISTANT:\nHello, user!\n\nASSISTANT:\nHello again, user!\n\n"
        );
        Ok(())
    }

    #[test]
    fn test_load() -> Result<()> {
        let buffer = Vec::<u8>::new();
        let mut sink = std::io::Cursor::new(buffer);
        let mut transcript = Transcript::new(&mut sink)?;

        let messages = vec![
            Message {
                role: Role::User,
                content: "Hello, assistant!".to_string(),
            },
            Message {
                role: Role::Assistant,
                content: "Hello, user!".to_string(),
            },
            Message {
                role: Role::Assistant,
                content: "Hello again, user!".to_string(),
            },
        ];
        for message in &messages {
            transcript.record(message)?;
        }

        sink.rewind()?;
        let loaded = load(sink)?;
        assert_eq!(loaded.len(), messages.len());
        for (message, loaded) in messages.iter().zip(loaded.iter()) {
            assert_eq!(message.role, loaded.role);
            assert_eq!(message.content, loaded.content);
        }
        Ok(())
    }
}
