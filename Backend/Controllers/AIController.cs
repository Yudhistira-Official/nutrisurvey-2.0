using Microsoft.AspNetCore.Mvc;
using NutritionApp.Models.AI;
using NutritionApp.Services;

namespace NutritionApp.Controllers
{
    [ApiController]
    [Route("api/[controller]")]
    public class AIController : ControllerBase
    {
        private readonly IAIService _aiService;
        private readonly IMealMappingService _mealMappingService;

        public AIController(IAIService aiService, IMealMappingService mealMappingService)
        {
            _aiService = aiService;
            _mealMappingService = mealMappingService;
        }

        [HttpPost("generate-menu")]
        public async Task<IActionResult> GenerateMenu([FromBody] AIIntegrationRequest request)
        {
            if (request == null)
            {
                return BadRequest(new { message = "Request body is required." });
            }

            if (request.TargetTDEE <= 0)
            {
                return BadRequest(new { message = "targetTDEE must be greater than 0." });
            }

            if (request.AIConfig == null)
            {
                return BadRequest(new { message = "aiConfig is required." });
            }

            if (string.IsNullOrWhiteSpace(request.AIConfig.Provider))
            {
                return BadRequest(new { message = "AI provider is required." });
            }

            if (string.IsNullOrWhiteSpace(request.AIConfig.Model))
            {
                return BadRequest(new { message = "AI model is required." });
            }

            if (string.IsNullOrWhiteSpace(request.AIConfig.BaseUrl))
            {
                return BadRequest(new { message = "AI baseUrl is required." });
            }

            if (RequiresApiKey(request.AIConfig.Provider) && string.IsNullOrWhiteSpace(request.AIConfig.ApiKey))
            {
                return BadRequest(new { message = "apiKey is required for this AI provider." });
            }

            try
            {
                var aiResponse = await _aiService.GetMealPlanFromAIAsync(request);
                var finalMenu = await _mealMappingService.MapAndCalculateAsync(aiResponse);
                NormalizeMenuToTdee(finalMenu, request.TargetTDEE);
                return Ok(finalMenu);
            }
            catch (Exception ex)
            {
                return StatusCode(500, new { message = ex.Message });
            }
        }

        private static bool RequiresApiKey(string provider)
        {
            return true;
        }

        private static void NormalizeMenuToTdee(List<MappedMealItem> menu, int targetTdee)
        {
            if (menu.Count == 0 || targetTdee <= 0)
            {
                return;
            }

            var totals = new
            {
                Calories = menu.Sum(item => item.Calories),
                Carbs = menu.Sum(item => item.Carbohydrate),
                Protein = menu.Sum(item => item.Protein),
                Fat = menu.Sum(item => item.Fat)
            };

            if (totals.Calories <= 0)
            {
                return;
            }

            var tolerance = targetTdee * 0.05;
            if (Math.Abs(totals.Calories - targetTdee) <= tolerance)
            {
                return;
            }

            var factor = (double)targetTdee / totals.Calories;
            var normalizedMenu = new List<MappedMealItem>();

            foreach (var item in menu)
            {
                var originalGrams = item.SuggestedGrams;
                if (originalGrams <= 0)
                {
                    continue;
                }

                var normalizedGrams = (int)Math.Round(originalGrams * factor, MidpointRounding.AwayFromZero);
                if (normalizedGrams < 25)
                {
                    continue;
                }

                var nutrientFactor = (double)normalizedGrams / originalGrams;
                item.SuggestedGrams = normalizedGrams;
                item.Calories = Math.Round(item.Calories * nutrientFactor, 2);
                item.Protein = Math.Round(item.Protein * nutrientFactor, 2);
                item.Fat = Math.Round(item.Fat * nutrientFactor, 2);
                item.Carbohydrate = Math.Round(item.Carbohydrate * nutrientFactor, 2);

                foreach (var nutrientName in item.Nutrients.Keys.ToList())
                {
                    item.Nutrients[nutrientName] = Math.Round(item.Nutrients[nutrientName] * nutrientFactor, 2);
                }

                normalizedMenu.Add(item);
            }

            menu.Clear();
            menu.AddRange(normalizedMenu);
        }
    }
}
