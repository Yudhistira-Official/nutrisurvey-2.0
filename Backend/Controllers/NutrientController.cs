using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using NutritionApp.Data;
using System.Linq;
using System.Threading.Tasks;

namespace NutritionApp.Controllers
{
    [ApiController]
    [Route("api/[controller]")]
    public class NutrientController : ControllerBase
    {
        private readonly AppDbContext _context;
        public NutrientController(AppDbContext context) { _context = context; }

        [HttpGet("list")]
        public async Task<IActionResult> GetNutrientList()
        {
            var list = await _context.Nutrients
                .Select(n => new { n.Name, n.Unit })
                .Distinct()
                .ToListAsync();
            return Ok(list);
        }
    }
}
