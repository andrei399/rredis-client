use std::u16;

use clap::{Parser, Subcommand};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Parser, Debug)]
#[command(author, version, about = "A simplified redis-like client.", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Get {
        key: String,
    },

    Set {
        key: String,
        value: String,
    },
    Setex {
        key: String,
        seconds: u64,
        value: String,
    },
}

async fn write_to_redis(mut client: TcpStream, message: &[u8]) -> io::Result<String> {
    client.write_all(message).await?;
    let mut buffer = [0u8; 1024];
    let n = client.read(&mut buffer).await?;

    let response = String::from_utf8_lossy(&buffer[..n]).into_owned();

    if response.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Redis server response is empty.",
        ));
    }

    match response.chars().next() {
        Some('+') => {
            let payload = &response[1..];
            Ok(payload.trim().to_string())
        }
        Some('-') => {
            let payload = &response[1..];
            eprintln!("Error {}", payload.trim());
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Redis server returned an error",
            ));
        }
        _ => {
            eprintln!("Unexpected response format: {}", response.trim());
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unexpected response format",
            ));
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Args::parse();
    let client = TcpStream::connect("127.0.0.1:6969").await?;

    let message = match cli.command {
        Commands::Get { key } => format!("GET {key}"),
        Commands::Set { key, value } => format!("SET {key} {value}"),
        Commands::Setex {
            key,
            seconds,
            value,
        } => format!("SETEX {key} {seconds} {value}"),
    };
    if let Some(response) = write_to_redis(client, message.as_bytes()).await.ok() {
        println!("{}", response);
    }
    Ok(())
}
