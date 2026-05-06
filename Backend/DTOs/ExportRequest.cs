using System.Collections.Generic;

namespace NutritionApp.DTOs
{
    public class FoodEntryDto
    {
        public string MealTime { get; set; } = "SARAPAN";
        public string Name { get; set; } = string.Empty;
        public double Amount { get; set; }
        public double ServingSize { get; set; } = 100;
        public string? ServingUnit { get; set; }
        public Dictionary<string, double> Nutrients { get; set; } = new Dictionary<string, double>();
    }

    public class ExportRequest
    {
        public List<FoodEntryDto> Foods { get; set; } = new List<FoodEntryDto>();
        public List<MealTimeDto>? MealTimes { get; set; }
        public NutritionTargetsDto? Targets { get; set; }
    }

    public class NutritionTargetsDto
    {
        public double Kcal { get; set; }
        public double Carbs { get; set; }
        public double Protein { get; set; }
        public double Fat { get; set; }
    }

    public class MealTimeDto
    {
        public string Id { get; set; } = string.Empty;
        public string Label { get; set; } = string.Empty;
    }

    public class FilterDto
    {
        public string Nutrient { get; set; } = string.Empty;
        public string Operator { get; set; } = "=";
        public double Value { get; set; }
    }
}
