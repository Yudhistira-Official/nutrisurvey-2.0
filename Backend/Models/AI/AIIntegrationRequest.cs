using System.Text.Json.Serialization;

namespace NutritionApp.Models.AI
{
    public class AIIntegrationRequest
    {
        [JsonPropertyName("targetTDEE")]
        public int TargetTDEE { get; set; }

        [JsonPropertyName("targetCarbs")]
        public int TargetCarbs { get; set; }

        [JsonPropertyName("targetProtein")]
        public int TargetProtein { get; set; }

        [JsonPropertyName("targetFat")]
        public int TargetFat { get; set; }

        [JsonPropertyName("availableMealTypes")]
        public List<string> AvailableMealTypes { get; set; } = new List<string>();

        [JsonPropertyName("aiConfig")]
        public AIConfiguration AIConfig { get; set; } = new AIConfiguration();
    }

    public class AIConfiguration
    {
        [JsonPropertyName("provider")]
        public string Provider { get; set; } = string.Empty;

        [JsonPropertyName("model")]
        public string Model { get; set; } = string.Empty;

        [JsonPropertyName("apiKey")]
        public string ApiKey { get; set; } = string.Empty;

        [JsonPropertyName("baseUrl")]
        public string BaseUrl { get; set; } = string.Empty;
    }
}
