using System;
using System.IO;

class Program {

    static async Task Run()
    {
        Console.WriteLine("Starting run.");
        var creds = AADCredential.LoadFromFile("../aad_credentials.json");
        var consumer = new SBConsumer{ Credential = creds, Namespace = "castle-rtestapp", QueueName = "testlog" };

        Console.WriteLine("Starting consumer.");
        var processor = await consumer.Start();

        while (true)
        {
            await Task.Delay(TimeSpan.FromSeconds(5));
        }
    }

    static void Main(string[] args)
    {
        Run().Wait();
    }
}
