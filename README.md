## Overview
**AI** + **R**ust.
This repo contains both library ([lib.rs](src/lib.rs) et al.) and binary ([main.rs](src/main.rs)) crates, providing both a simple CLI and generic backend for interfacing with textual AI models 
over an OpenAI API-like interface. 

```bash
export API_KEY='your-api-key'
air -n 'gpt-4' -i 'context.txt' -o 'transcript.txt'
```
A REPL-like environment starts, allow you to submit commands to the selected 
model:
```text
OpenAI gpt-4  (air v0.1.1)
>> What is the meaning of life?

As an AI, I don't have personal thoughts or feelings. However, I can tell you that interpretations about the meaning of life can vary greatly depending on cultural, religious, philosophical, or personal beliefs. Some people may believe it's to learn and grow, while others may see it as serving others, seeking happiness, or contributing to a larger community or societal progression. It's a deeply personal and subjective topic. 
```

If the `output` flag is provided, the transcript is logged in real-time:
```bash 
cat transcript.txt

# USER:
# What is the meaning of life?

# ASSISTANT:
# As an AI, I don't have personal experiences or beliefs. However, ... 
```

If the `input` flag is provided with a saved transcript file, the 
conversation will continue where the previous left off.

```bash
air -n 'gpt-4' -i 'transcript.txt'
```
```
>> What were we discussing again?

We were discussing the meaning of life.
```

## Install
### From source
With a recent Rust toolchain and the git CLI:
```bash
git clone https://github.com/jm2534/air.git
cargo build --release
```

The release binary (by default at `./target/release/air`) can then be invoked. On Linux, move 
the binary to `/usr/local/bin` to give command line access as seen in the above examples.

## Usage
### Sending Requests
The `Client` struct allows for interfacing with arbitrary model "providers", such as OpenAI,
Anthropic (TODO), and local models. 
`Client`s maintain a context for each conversation, sending messages to the provider for processing. See the `Provider` trait for more information on implementing a model provider.

```rust
use air::Message;
use air::client::Client;
use air::host::OpenAI;

let model = OpenAI::new("gpt-3.5-turbo", "my-api-key");
let mut client = Client::new(model);
let message = Message::user("What is the meaning of life?");
let answer = client.send(message);
```

### Transcribing Conversations
Transcription can be useful for audting model performance, continuing long-running conversations, and general offline review. Rather than strictly require a file-based interface, however, transcription is genericized over I/O sources using Rust's powerful [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) and [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html) traits. For example, the `Transcript` struct has following simplified signature:

```rust
pub struct Transcript<'a, T: Write> {
    sink: Option<&'a mut T>,
    // ...
}
```

Here, `sink` can be anything that is `Write`: sockets, pipes, cloud blobs, and of 
course local files to name only a few. Similarly, the `transcript::load` method
requires only that it's `source` is `Read`. Any `Read` that was populated via 
`Transcript.record` will work:

```rust
pub fn load(source: impl Read) -> Result<Vec<Message>>
```