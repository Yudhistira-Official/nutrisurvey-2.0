using Microsoft.EntityFrameworkCore;
using NutritionApp.Data;
using NutritionApp.Models;
using NutritionApp.Models.AI;

namespace NutritionApp.Services
{
    public class MealMappingService : IMealMappingService
    {
        private readonly AppDbContext _context;

        public MealMappingService(AppDbContext context)
        {
            _context = context;
        }

        public async Task<List<MappedMealItem>> MapAndCalculateAsync(AIPromptResponse aiResponse)
        {
            var mappedMeals = new List<MappedMealItem>();

            foreach (var aiMeal in aiResponse.MealPlan)
            {
                var keyword = Normalize(aiMeal.FoodKeyword);
                if (string.IsNullOrWhiteSpace(keyword) || aiMeal.SuggestedGrams <= 0)
                {
                    continue;
                }

                var food = await FindBestFoodMatchAsync(keyword);
                if (food == null)
                {
                    continue;
                }

                mappedMeals.Add(MapFood(aiMeal, food));
            }

            return mappedMeals;
        }

        private async Task<Food?> FindBestFoodMatchAsync(string normalizedKeyword)
        {
            var candidates = await _context.Foods
                .Include(f => f.FoodNutrients)
                .ThenInclude(fn => fn.Nutrient)
                .Where(f => f.Name.ToLower().Contains(normalizedKeyword))
                .Take(25)
                .ToListAsync();

            return candidates
                .OrderBy(f => GetMatchScore(Normalize(f.Name), normalizedKeyword))
                .ThenBy(f => f.Name.Length)
                .FirstOrDefault();
        }

        private static MappedMealItem MapFood(AIMealItem aiMeal, Food food)
        {
            var referenceGrams = food.ServingSize > 0 ? food.ServingSize : 100.0;
            var multiplier = (double)aiMeal.SuggestedGrams / referenceGrams;
            var nutrients = new Dictionary<string, double>();

            foreach (var foodNutrient in food.FoodNutrients)
            {
                var nutrientName = foodNutrient.Nutrient.Name.ToLower();
                var amount = Math.Round(foodNutrient.Amount * multiplier, 2);
                nutrients[nutrientName] = amount;
            }

            return new MappedMealItem
            {
                MealType = aiMeal.MealType,
                RequestedKeyword = aiMeal.FoodKeyword,
                MatchedFoodId = food.Id,
                MatchedFoodName = food.Name,
                SuggestedGrams = aiMeal.SuggestedGrams,
                ReferenceGrams = referenceGrams,
                Calories = GetNutrientValue(nutrients, "energi", "energy", "calories", "kcal"),
                Protein = GetNutrientValue(nutrients, "protein"),
                Fat = GetNutrientValue(nutrients, "lemak total", "lemak", "fat", "total fat"),
                Carbohydrate = GetNutrientValue(nutrients, "karbohidrat total", "karbohidrat", "carbohydrate", "carbs", "total carbohydrate"),
                Nutrients = nutrients,
                Reasoning = aiMeal.Reasoning
            };
        }

        private static string Normalize(string value)
        {
            return string.Join(' ', value.Trim().ToLower().Split(' ', StringSplitOptions.RemoveEmptyEntries));
        }

        private static int GetMatchScore(string foodName, string keyword)
        {
            if (foodName == keyword) return 0;
            if (foodName.StartsWith(keyword)) return 1;
            if (foodName.Contains(keyword)) return 2;
            return 3;
        }

        private static double GetNutrientValue(Dictionary<string, double> nutrients, params string[] aliases)
        {
            foreach (var alias in aliases)
            {
                if (nutrients.TryGetValue(alias, out var exactValue))
                {
                    return exactValue;
                }

                var partialMatch = nutrients.FirstOrDefault(n => n.Key.Contains(alias));
                if (!string.IsNullOrWhiteSpace(partialMatch.Key))
                {
                    return partialMatch.Value;
                }
            }

            return 0;
        }
    }
}
