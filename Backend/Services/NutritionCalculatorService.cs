using NutritionApp.DTOs;

namespace NutritionApp.Services
{
    public interface INutritionCalculatorService
    {
        TdeeResponse CalculateTdee(TdeeRequest request);
    }

    public class NutritionCalculatorService : INutritionCalculatorService
    {
        public TdeeResponse CalculateTdee(TdeeRequest request)
        {
            double bmr;
            if (request.Gender.Equals("Male", StringComparison.OrdinalIgnoreCase))
            {
                // Harris-Benedict for Men: 66 + (13.7 * weight) + (5 * height) - (6.8 * age)
                bmr = 66 + (13.7 * request.WeightKg) + (5 * request.HeightCm) - (6.8 * request.Age);
            }
            else
            {
                // Harris-Benedict for Women: 655 + (9.6 * weight) + (1.8 * height) - (4.7 * age)
                bmr = 655 + (9.6 * request.WeightKg) + (1.8 * request.HeightCm) - (4.7 * request.Age);
            }

            return new TdeeResponse
            {
                BasalMetabolicRate = Math.Round(bmr, 2),
                TotalDailyEnergyExpenditure = Math.Round(bmr * request.ActivityFactor * request.InjuryFactor, 2),
                FormulaUsed = "Harris-Benedict (Clinical Edition)"
            };
        }
    }
}
