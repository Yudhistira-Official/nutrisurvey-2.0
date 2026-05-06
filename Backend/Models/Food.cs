using System.ComponentModel.DataAnnotations;

namespace NutritionApp.Models
{
    public class Food
    {
        [Key]
        public int Id { get; set; }
        [Required]
        public string Name { get; set; } = string.Empty;
        public string? Brand { get; set; }
        public string? Category { get; set; }
        
        // Base serving information for calculations
        public double ServingSize { get; set; } = 100; // Default to 100
        public string ServingUnit { get; set; } = "g";  // Default to g
        public double ServingsPerContainer { get; set; } = 1;

        public virtual ICollection<FoodNutrient> FoodNutrients { get; set; } = new List<FoodNutrient>();
    }
}
