using Microsoft.EntityFrameworkCore;
using NutritionApp.Models;

namespace NutritionApp.Data
{
    public class AppDbContext : DbContext
    {
        public AppDbContext(DbContextOptions<AppDbContext> options) : base(options) { }

        public DbSet<Food> Foods { get; set; } = null!;
        public DbSet<Nutrient> Nutrients { get; set; } = null!;
        public DbSet<FoodNutrient> FoodNutrients { get; set; } = null!;

        protected override void OnModelCreating(ModelBuilder modelBuilder)
        {
            base.OnModelCreating(modelBuilder);
            // Additional configuration if needed
        }
    }
}
