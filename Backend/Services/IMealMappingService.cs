using NutritionApp.Models.AI;

namespace NutritionApp.Services
{
    public interface IMealMappingService
    {
        Task<List<MappedMealItem>> MapAndCalculateAsync(AIPromptResponse aiResponse);
    }
}
