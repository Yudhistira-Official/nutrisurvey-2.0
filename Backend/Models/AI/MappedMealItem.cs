using System.Text.Json.Serialization;

namespace NutritionApp.Models.AI
{
    public class MappedMealItem
    {
        [JsonPropertyName("meal_type")]
        public string MealType { get; set; } = string.Empty;

        [JsonPropertyName("requested_keyword")]
        public string RequestedKeyword { get; set; } = string.Empty;

        [JsonPropertyName("matched_food_id")]
        public int MatchedFoodId { get; set; }

        [JsonPropertyName("matched_food_name")]
        public string MatchedFoodName { get; set; } = string.Empty;

        [JsonPropertyName("suggested_grams")]
        public int SuggestedGrams { get; set; }

        [JsonPropertyName("reference_grams")]
        public double ReferenceGrams { get; set; }

        [JsonPropertyName("calories")]
        public double Calories { get; set; }

        [JsonPropertyName("protein")]
        public double Protein { get; set; }

        [JsonPropertyName("fat")]
        public double Fat { get; set; }

        [JsonPropertyName("carbohydrate")]
        public double Carbohydrate { get; set; }

        [JsonPropertyName("nutrients")]
        public Dictionary<string, double> Nutrients { get; set; } = new Dictionary<string, double>();

        [JsonPropertyName("reasoning")]
        public string Reasoning { get; set; } = string.Empty;
    }
}
