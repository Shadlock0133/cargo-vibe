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

#[tokio::main]
async fn main() {
    match Opt::parse() {
        Opt::Vibe(Cmd::Build) => {
            let client = spawn(start_client());
            let cmd = Command::new("cargo")
                .args(&[
                    "build",
                    "--message-format=json-diagnostic-rendered-ansi,\
                        json-render-diagnostics",
                ])
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let output = cmd.wait_with_output().unwrap();
            if is_success(output.stdout) {
                eprintln!("[cargo-vibe] build successful!");
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
                eprintln!("[cargo-vibe] build failed");
            }
        }
    }
}
