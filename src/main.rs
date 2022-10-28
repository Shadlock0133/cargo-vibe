use std::{
    io::BufReader,
    process::{Command, Stdio},
    time::Duration,
};

use buttplug::{client::ButtplugClient, util::in_process_client};
use cargo_metadata::{
    diagnostic::Diagnostic, BuildFinished, CompilerMessage, Message,
};
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
    Check,
}

async fn start_client() -> ButtplugClient {
    let client = in_process_client("cargo-vibe", false).await;
    client.start_scanning().await.unwrap();
    sleep(Duration::from_secs(1)).await;
    client.stop_scanning();
    client
}

async fn vibrate_all(client: &ButtplugClient, speed: f64, duration: Duration) {
    for device in client.devices() {
        device
            .vibrate(&buttplug::client::VibrateCommand::Speed(speed))
            .await
            .unwrap();
    }
    sleep(duration).await;
    client.stop_all_devices().await.unwrap();
}

fn is_success(stdout: Vec<u8>) -> bool {
    for message in Message::parse_stream(BufReader::new(stdout.as_slice())) {
        match message {
            Ok(Message::BuildFinished(BuildFinished { success, .. })) => {
                return success
            }
            Ok(Message::CompilerMessage(CompilerMessage {
                message:
                    Diagnostic {
                        rendered: Some(rendered),
                        ..
                    },
                ..
            })) => {
                eprintln!("{rendered}");
            }
            _ => (),
        }
    }
    false
}

const CARGO_JSON_FLAG: &str =
    "--message-format=json-diagnostic-rendered-ansi,json-render-diagnostics";

#[tokio::main]
async fn main() {
    let client = spawn(start_client());
    match Opt::parse() {
        Opt::Vibe(Cmd::Build) => {
            let cmd = Command::new("cargo")
                .args(&["build", CARGO_JSON_FLAG])
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let output = cmd.wait_with_output().unwrap();
            if is_success(output.stdout) {
                eprintln!("[cargo-vibe] build successful!");
                let client = client.await.unwrap();
                vibrate_all(&client, 1.0, Duration::from_secs(3)).await;
            } else {
                eprintln!("[cargo-vibe] build failed");
            }
        }
        Opt::Vibe(Cmd::Check) => {
            let cmd = Command::new("cargo")
                .args(&["check", CARGO_JSON_FLAG])
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let output = cmd.wait_with_output().unwrap();
            if is_success(output.stdout) {
                eprintln!("[cargo-vibe] check successful!");
                let client = client.await.unwrap();
                vibrate_all(&client, 1.0, Duration::from_secs(3)).await;
            } else {
                eprintln!("[cargo-vibe] check failed");
            }
        }
    }
}
