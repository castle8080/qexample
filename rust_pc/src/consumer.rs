use std::error::Error;

use chrono::Local;

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

pub async fn run_consumer(sb_client: &AzureServiceBusClient) -> Result<(), Box<dyn Error>> {
    match sb_client.peek_lock().await? {
        None => {
            println!("No message found.");
        },
        Some(msg) => {
            println!("Recieved message:");
            println!("    properties: {:?}", msg.properties);

            let payload: LogInfo = msg.json_into()?;
            println!("    content: {:?}", payload);

            sb_client.delete_message(&msg.properties).await?;
        }
    }

    Ok(())
}
