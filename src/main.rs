use std::{io::BufReader, process::Command, time::Duration};

use buttplug::{client::ButtplugClient, util::in_process_client};
use cargo_metadata::{BuildFinished, Message};
use clap::{Parser, Subcommand};
use tokio::{spawn, time::sleep};

#[derive(Parser)]
enum Opt {
    #[clap(subcommand)]
    Vibe(Cmd),
}

#[derive(Subcommand)]
enum Cmd {
    Build,
}

async fn start_client() -> ButtplugClient {
    let client = in_process_client("cargo-vibe", false).await;
    client.start_scanning().await.unwrap();
    sleep(Duration::from_secs(1)).await;
    client.stop_scanning();
    client
}

fn is_success(stdout: Vec<u8>) -> bool {
    for message in Message::parse_stream(BufReader::new(stdout.as_slice())) {
        if let Ok(Message::BuildFinished(BuildFinished {
            success: true, ..
        })) = message
        {
            return true;
        }
    }
    false
}

#[tokio::main]
async fn main() {
    match Opt::parse() {
        Opt::Vibe(Cmd::Build) => {
            let client = spawn(start_client());
            let output = Command::new("cargo")
                .args(&["build", "--message-format=json"])
                .output()
                .unwrap();

            if is_success(output.stdout) {
                eprintln!("build successful!");
                let client = client.await.unwrap();
                for device in client.devices() {
                    device
                        .vibrate(&buttplug::client::VibrateCommand::Speed(1.0))
                        .await
                        .unwrap();
                }
                sleep(Duration::from_secs(3)).await;
                client.stop_all_devices().await.unwrap();
            } else {
                eprintln!(
                    "build failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}
