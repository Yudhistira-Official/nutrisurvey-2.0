using System.Text.Json.Serialization;

namespace NutritionApp.Models.AI
{
    public class AIPromptResponse
    {
        [JsonPropertyName("meal_plan")]
        public List<AIMealItem> MealPlan { get; set; } = new List<AIMealItem>();
    }

    public class AIMealItem
    {
        [JsonPropertyName("meal_type")]
        public string MealType { get; set; } = string.Empty;

        [JsonPropertyName("food_keyword")]
        public string FoodKeyword { get; set; } = string.Empty;

        [JsonPropertyName("suggested_grams")]
        public int SuggestedGrams { get; set; }

        [JsonPropertyName("reasoning")]
        public string Reasoning { get; set; } = string.Empty;
    }
}
