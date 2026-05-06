using System.ComponentModel.DataAnnotations;

namespace NutritionApp.Models
{
    public class Nutrient
    {
        [Key]
        public int Id { get; set; }
        [Required]
        public string Name { get; set; } = string.Empty;
        [Required]
        public string Unit { get; set; } = string.Empty; // e.g., g, mg, kcal
    }
}
