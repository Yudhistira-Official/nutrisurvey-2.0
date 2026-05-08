using NutritionApp.Models.AI;

namespace NutritionApp.Services
{
    public interface IAIService
    {
        Task<AIPromptResponse> GetMealPlanFromAIAsync(AIIntegrationRequest request);
    }
}
