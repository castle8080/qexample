# Q Example

This is an example using Auzre Service Bus from Rust. In this example, I am not using the azure sdk and trying to use the REST APIs directly. This is also done for me to learn the interactions from the API perspective. This is mostly a learning/example project.

## Setup Requirements

A AAD application (Microsoft Entra ID now) needs to be created for the app to authenticate. A file called aad_credentials shold be created with the values. See aad_credentials_template.json to see what fields need to be filled out.

An Azure Servie Bus queue also needs to be created. Change the run_producer and run_consumer scripts for the queue and service bus names.
