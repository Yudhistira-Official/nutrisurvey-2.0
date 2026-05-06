using Microsoft.AspNetCore.Mvc;
using NutritionApp.DTOs;
using NutritionApp.Services;

namespace NutritionApp.Controllers
{
    [ApiController]
    [Route("api/[controller]")]
    public class NutritionController : ControllerBase
    {
        private readonly INutritionCalculatorService _calculatorService;

        public NutritionController(INutritionCalculatorService calculatorService)
        {
            _calculatorService = calculatorService;
        }

        [HttpPost("calculate-tdee")]
        public IActionResult CalculateTdee([FromBody] TdeeRequest request)
        {
            if (request == null) return BadRequest("Invalid request data.");

            var result = _calculatorService.CalculateTdee(request);
            return Ok(result);
        }
    }
}
