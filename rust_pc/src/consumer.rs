use std::error::Error;

use chrono::Local;
use tokio::time::{Duration, sleep};

use crate::mazure::sbclient::AzureServiceBusClient;
use crate::messages::LogInfo;

pub async fn run_consumer_loop(sb_client: &AzureServiceBusClient) -> Result<(), Box<dyn Error>> {
    loop {
        println!("[{}] Waiting for new message.", Local::now());
        match run_consumer(sb_client).await {
            Err(e) => {
                println!("Error processing: {:?}", e);
            },
            Ok(_) => {
                // Do nothing!
            }
        }
    }
}

pub async fn process_message(log_info: LogInfo) {
    let start = Local::now();
    let end = start + chrono::Duration::seconds(30);

    while Local::now() < end {
        println!("[{}] Still processing.....", Local::now());
        sleep(Duration::from_secs(5)).await;
    }

    println!("[{}] ok processed: {:?}", Local::now(), &log_info);
}

pub async fn run_consumer(sb_client: &AzureServiceBusClient) -> Result<(), Box<dyn Error>> {
    match sb_client.peek_lock().await? {
        None => {
            println!("No message found.");
        },
        Some(msg) => {
            println!("Recieved message: time={}", Local::now());
            println!("    properties: {:?}", msg.properties);

            let payload: LogInfo = msg.json_into()?;
            println!("    content: {:?}", payload);

            match msg.properties.delivery_count {
                None => {
                    println!("No sequence number found");
                    sb_client.delete_message(&msg.properties).await?;
                },
                Some(1) => {
                    println!("First seen - unlock it.");
                    sb_client.unlock_message(&msg.properties).await?;
                },
                Some(2) => {
                    println!("2nd time seen - leave the lock.");
                    sb_client.renew_lock(&msg.properties).await?;
                },
                Some(_) => {
                    println!("Processing now:");
                    let t = process_message(payload);
                    t.await;

                    println!("Ok its processed now");
                    sb_client.delete_message(&msg.properties).await?;
                }
            }
        }
    }

    Ok(())
}
