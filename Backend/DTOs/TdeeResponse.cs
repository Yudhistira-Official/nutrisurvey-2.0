namespace NutritionApp.DTOs
{
    public class TdeeResponse
    {
        public double BasalMetabolicRate { get; set; }
        public double TotalDailyEnergyExpenditure { get; set; }
        public string FormulaUsed { get; set; } = "Harris-Benedict (Clinical Edition)";
    }
}
