using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using NutritionApp.Data;
using NutritionApp.DTOs;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace NutritionApp.Controllers
{
    [ApiController]
    [Route("api/[controller]")]
    public class FoodSearchController : ControllerBase
    {
        private readonly AppDbContext _context;

        public FoodSearchController(AppDbContext context)
        {
            _context = context;
        }

        [HttpGet("search")]
        public async Task<IActionResult> SearchByName([FromQuery] string? query)
        {
            var dbQuery = _context.Foods
                .Include(f => f.FoodNutrients)
                .ThenInclude(fn => fn.Nutrient)
                .AsQueryable();

            if (!string.IsNullOrWhiteSpace(query))
            {
                dbQuery = dbQuery.Where(f => f.Name.ToLower().Contains(query.ToLower()));
            }
            else 
            {
                dbQuery = dbQuery.OrderBy(f => f.Name).Take(5);
            }

            var rawResults = await dbQuery.Take(20).ToListAsync();

            var results = rawResults.Select(f => new {
                f.Id,
                f.Name,
                f.Brand,
                f.Category,
                f.ServingSize,
                f.ServingUnit,
                f.ServingsPerContainer,
                Nutrients = f.FoodNutrients.ToDictionary(
                    fn => fn.Nutrient.Name.ToLower(), 
                    fn => fn.Amount
                )
            }).ToList();

            return Ok(results);
        }

        [HttpGet("status")]
        public async Task<IActionResult> GetDatabaseStatus()
        {
            var foodCount = await _context.Foods.CountAsync();
            return Ok(new { foodCount, isReady = foodCount > 0 });
        }

        [HttpPost("recommendations")]
        public async Task<IActionResult> GetRecommendations([FromBody] List<FilterDto> filters)
        {
            if (filters == null || filters.Count == 0) return BadRequest("Filter diperlukan.");

            IQueryable<NutritionApp.Models.Food> query = _context.Foods
                .Include(f => f.FoodNutrients)
                .ThenInclude(fn => fn.Nutrient);

            foreach (var filter in filters)
            {
                var nutrientName = filter.Nutrient.ToLower();
                
                query = filter.Operator switch
                {
                    ">" => query.Where(f => f.FoodNutrients.Any(fn => (fn.Nutrient.Name.ToLower().Contains(nutrientName)) && fn.Amount > filter.Value)),
                    "<" => query.Where(f => f.FoodNutrients.Any(fn => (fn.Nutrient.Name.ToLower().Contains(nutrientName)) && fn.Amount < filter.Value)),
                    "=" => query.Where(f => f.FoodNutrients.Any(fn => (fn.Nutrient.Name.ToLower().Contains(nutrientName)) && fn.Amount == filter.Value)),
                    _ => query
                };
            }

            var results = await query.Take(10).ToListAsync();
            
            var finalResults = results.Select(f => new {
                f.Id,
                f.Name,
                f.Brand,
                f.Category,
                f.ServingSize,
                f.ServingUnit,
                f.ServingsPerContainer,
                Nutrients = f.FoodNutrients.ToDictionary(fn => fn.Nutrient.Name.ToLower(), fn => fn.Amount)
            });

            return Ok(finalResults);
        }
    }
}
