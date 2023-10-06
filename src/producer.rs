use std::error::Error;
use crate::mazure::sbclient::AzureServiceBusClient;
use crate::messages::LogInfo;

pub async fn run_producer(sb_client: &AzureServiceBusClient, count: u32) -> Result<(), Box<dyn Error>> {
    for _ in 1..count {
        let msg = LogInfo::new_random();
        println!("Sending message: {:?}", msg);
        sb_client.send_json(&msg).await?;
        println!("Message sent!");
    }
    Ok(())
}