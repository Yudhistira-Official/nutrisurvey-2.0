using System.ComponentModel.DataAnnotations;

namespace NutritionApp.Models
{
    public class FoodNutrient
    {
        [Key]
        public int Id { get; set; }

        [Required]
        public int FoodId { get; set; }
        public virtual Food Food { get; set; } = null!;

        [Required]
        public int NutrientId { get; set; }
        public virtual Nutrient Nutrient { get; set; } = null!;

        [Required]
        public double Amount { get; set; } // Amount of nutrient per 100g usually
    }
}
