
using System.Runtime.CompilerServices;
using Newtonsoft.Json;

public class AADCredential {

    [JsonProperty("tenant_id")]
    public string TenantId { get; private set; }

    [JsonProperty("client_id")]
    public string ClientId { get; private set; }

    [JsonProperty("secret")]
    public string Secret { get; private set; }

    [JsonConstructor()]
    public AADCredential(
        [JsonProperty("tenant_id")] string tenantId,
        [JsonProperty("client_id")] string clientId,
        [JsonProperty("secret")] string secret)
    {
        this.TenantId = tenantId;
        this.ClientId = clientId;
        this.Secret = secret;
    }

    public override string ToString()
    {
        return $"AADCredential: TenantId={TenantId} ClientId={ClientId}";
    }

    public static AADCredential LoadFromFile(String filePath) {
        using (FileStream fileStream = File.OpenRead(filePath))
        using (StreamReader streamReader = new StreamReader(fileStream))
        using (JsonTextReader jsonReader = new JsonTextReader(streamReader))
        {
            var serializer = new JsonSerializer();
            return serializer.Deserialize<AADCredential>(jsonReader);
        }
    }
}