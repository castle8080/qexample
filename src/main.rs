mod mazure;
mod messages;
mod producer;
mod consumer;

use std::error::Error;
use std::sync::Arc;

use clap::Parser;
use mazure::aadclient::{AADClient, AADCredentials};
use mazure::sbclient::{SERVICE_BUS_RESOURCE, AzureServiceBusClient};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug)]
enum Mode {
    Producer,
    Consumer
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CommandLineArgs {
    #[arg(short = 'm', long = "mode", )]
    mode: Mode,

    #[arg(short = 'c', long = "credentials", )]
    credentials_file: String,

    #[arg(short = 'n', long = "namespace", )]
    service_bus_namespace: String,

    #[arg(short = 'q', long = "queue", )]
    queue: String,

    #[arg(long = "count", default_value = "1", )]
    count: u32,
}

impl CommandLineArgs {
    fn create_sb_client(self: &Self) -> Result<AzureServiceBusClient, Box<dyn Error>> {
        let aad_creds = AADCredentials::from_file(&self.credentials_file)?;

        let http_client = reqwest::Client::new();

        let aad_client = Arc::new(AADClient::new(http_client.clone(), aad_creds, SERVICE_BUS_RESOURCE, Option::None));
        Ok(AzureServiceBusClient::new(aad_client.clone(), http_client, &self.service_bus_namespace, &self.queue))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let args = CommandLineArgs::parse();
    let sb_client = args.create_sb_client()?;

    match args.mode {
        Mode::Consumer => {
            consumer::run_consumer_loop(&sb_client).await?;
        },
        Mode::Producer => {
            producer::run_producer(&sb_client, args.count).await?;
        }
    }

    Ok(())
}
