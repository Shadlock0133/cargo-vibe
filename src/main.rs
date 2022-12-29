use std::{ffi::OsStr, time::Duration};

use buttplug::{
    client::{ButtplugClient, ButtplugClientError, VibrateCommand},
    core::{
        connector::{
            ButtplugRemoteConnector as RemoteConn,
            ButtplugWebsocketClientTransport as WebSocketTransport,
        },
        message::serializer::ButtplugClientJSONSerializer as JsonSer,
    },
    util::in_process_client,
};
use futures::FutureExt;
use tokio::{spawn, time::sleep};

const CLIENT_NAME: &str = "cargo-vibe";

async fn connect_to_server() -> Result<ButtplugClient, ButtplugClientError> {
    let client = ButtplugClient::new(CLIENT_NAME);
    let connector = RemoteConn::<_, JsonSer, _, _>::new(
        WebSocketTransport::new_insecure_connector("ws://127.0.0.1:12345"),
    );
    client.connect(connector).await?;
    client.start_scanning().await?;
    Ok(client)
}

async fn start_in_process_server() -> Result<ButtplugClient, ButtplugClientError>
{
    let client = in_process_client(CLIENT_NAME, false).await;
    client.start_scanning().await?;
    Ok(client)
}

// Parses pattern like "0.5 3s/0.75 1.5s"
fn parse_pattern(
    pattern: &str,
) -> Result<Vec<(f64, Duration)>, Box<dyn std::error::Error>> {
    pattern
        .split('/')
        .map(|x| {
            let (speed, duration) =
                x.split_once(' ').ok_or("couldn't split")?;
            let speed = speed.parse()?;
            let duration = duration
                .strip_suffix('s')
                .ok_or("missing 's'")?
                .parse()
                .map(Duration::from_secs_f64)?;
            Ok((speed, duration))
        })
        .collect()
}

async fn vibrate_all(
    client: &ButtplugClient,
) -> Result<(), ButtplugClientError> {
    let pattern = std::env::var("CARGO_VIBE_PATTERN")
        .ok()
        .as_deref()
        .and_then(|x| {
            parse_pattern(x)
                .map_err(|e| eprintln!("pattern error: {e}"))
                .ok()
        })
        .unwrap_or_else(|| vec![(1.0, Duration::from_secs(3))]);
    eprintln!("{pattern:?}");

    let devices = client.devices();
    if !devices.is_empty() {
        for (speed, duration) in pattern {
            for device in &devices {
                device.vibrate(&VibrateCommand::Speed(speed)).await?;
            }
            sleep(duration).await;
        }
        client.stop_all_devices().await?;
    } else {
        eprintln!("[cargo-vibe] no devices found");
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let code = real_main().await.unwrap_or_else(|e| {
        eprintln!("Error: {:?}", e);
        -1
    });
    std::process::exit(code)
}

// code stolen from cargo-mommy, thanks Gankra
async fn real_main() -> Result<i32, Box<dyn std::error::Error>> {
    let remote_client = spawn(connect_to_server());
    let in_process_client = spawn(start_in_process_server());

    let cargo_var = std::env::var_os("CARGO");
    let cargo = cargo_var.as_deref().unwrap_or(OsStr::new("cargo"));
    let mut arg_iter = std::env::args_os();
    let _cargo = arg_iter.next();
    let _cmd = arg_iter.next();

    let status = std::process::Command::new(cargo).args(arg_iter).status()?;
    if status.success() {
        eprintln!("[cargo-vibe] success!");
        // get remote client, or fallback to in-process one
        let client = if let Some(Ok(client)) = remote_client.now_or_never() {
            eprintln!("[cargo-vibe] using server");
            Ok(client)
        } else {
            eprintln!("[cargo-vibe] starting in-process server");
            in_process_client.await
        };
        if let Ok(Ok(client)) = client {
            if let Err(e) = vibrate_all(&client).await {
                eprintln!("[cargo-vibe] error trying to vibe: {e}")
            }
        } else {
            eprintln!("[cargo-vibe] sorry, couldn't create a client")
        }
    } else {
        eprintln!("[cargo-vibe] failed");
    }
    Ok(status.code().unwrap_or(-1))
}
