use std::error::Error;

use chrono::Local;

use crate::mazure::sbclient::{AzureServiceBusClient, Message};
use crate::messages::LogInfo;

pub async fn run_producer(sb_client: &AzureServiceBusClient, count: u32) -> Result<(), Box<dyn Error>> {
    for _ in 1..=count {
        let log_info = LogInfo::new_random();
        let mut msg = Message::new_json(&log_info)?;

        // Set the message to be processed in the future.
        let eq_time = chrono::Utc::now() + chrono::Duration::seconds(15);
        msg.properties.scheduled_enqueue_time_utc = Some(eq_time);

        println!("[{}] Sending message:", Local::now());
        println!("    properties: {:?}", &msg.properties);
        println!("    content: {:?}", &log_info);

        sb_client.send(&msg).await?;
        println!("[{}] Message sent!", Local::now());
    }
    Ok(())
}