using Azure.Identity;
using Azure.Messaging.ServiceBus;

public class SBConsumer {
    public AADCredential Credential { get; set; }

    public string Namespace { get; set; }

    public string QueueName { get; set; }

    public async Task<ServiceBusProcessor> Start()
    {
        var credential = new ClientSecretCredential(
            Credential.TenantId,
            Credential.ClientId,
            Credential.Secret);

        var client = new ServiceBusClient($"{this.Namespace}.servicebus.windows.net", credential);

        // Make marking completion to be explicit in the processor callback.
        var serviceBusProcessor = client.CreateProcessor(this.QueueName, new ServiceBusProcessorOptions {
            AutoCompleteMessages = false,
            ReceiveMode = ServiceBusReceiveMode.PeekLock,
            MaxConcurrentCalls = 1,
            MaxAutoLockRenewalDuration = TimeSpan.FromMinutes(2)
        });

        serviceBusProcessor.ProcessMessageAsync += this.Process;
        serviceBusProcessor.ProcessErrorAsync += this.ProcessError;

        await serviceBusProcessor.StartProcessingAsync();
        return serviceBusProcessor;
    }

    private async Task Process(ProcessMessageEventArgs messageEvent)
    {
        Console.WriteLine($"[{DateTime.Now}] Received message: id={messageEvent.Message.CorrelationId} count={messageEvent.Message.DeliveryCount}");
        try {
            var content = messageEvent.Message.Body.ToString();

            Console.WriteLine($"Content: {content}");

            if (messageEvent.Message.DeliveryCount == 1) {
                Console.WriteLine("Abandon message.");
                // Should get the message back right away.
                await messageEvent.AbandonMessageAsync(messageEvent.Message);
            }
            else if (messageEvent.Message.DeliveryCount == 2) {
                Console.WriteLine("Do nothing with the message.");
                // This should leave the lock and keep someone from picking it up until the lock expires.
            }
            else {
                await ProcessWithWait(messageEvent);
                Console.WriteLine("Complete the message");
                await messageEvent.CompleteMessageAsync(messageEvent.Message);
            }
        }
        catch (Exception e) {
            Console.WriteLine($"Error processing: {e}");
            await messageEvent.AbandonMessageAsync(messageEvent.Message);
        }

        return;
    }

    private async Task ProcessWithWait(ProcessMessageEventArgs messageEvent)
    {
        var start = DateTime.UtcNow;
        var stopTime = start + TimeSpan.FromMinutes(4);

        while (DateTime.UtcNow < stopTime) {
            Console.WriteLine($"[{DateTime.UtcNow}] Still processing.");
            await Task.Delay(TimeSpan.FromSeconds(15));
        }

        Console.WriteLine($"[{DateTime.UtcNow}] Done.");
    }

    private Task ProcessError(ProcessErrorEventArgs errorEvent)
    {
        Console.WriteLine($"Received message: error={errorEvent}");
        return Task.CompletedTask;
    }
}