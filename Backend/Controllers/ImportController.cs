using Microsoft.AspNetCore.Mvc;
using NutritionApp.Services;

namespace NutritionApp.Controllers
{
    [ApiController]
    [Route("api/[controller]")]
    public class ImportController : ControllerBase
    {
        private readonly ICsvImportService _importService;

        public ImportController(ICsvImportService importService)
        {
            _importService = importService;
        }

        [HttpPost("food-db")]
        public async Task<IActionResult> ImportFoodDb(IFormFile file)
        {
            if (file == null || file.Length == 0) return BadRequest("File tidak valid.");

            try
            {
                // Ensure directory exists
                var dbDirectory = Path.Combine(Directory.GetCurrentDirectory(), "Data", "MasterDatabases");
                if (!Directory.Exists(dbDirectory)) Directory.CreateDirectory(dbDirectory);

                // Save copy for persistence
                var filePath = Path.Combine(dbDirectory, $"{DateTime.Now:yyyyMMddHHmmss}_{file.FileName}");
                using (var stream = new FileStream(filePath, FileMode.Create))
                {
                    await file.CopyToAsync(stream);
                }

                var count = await _importService.ImportFoodDatabaseAsync(file);
                return Ok(new { Message = $"Berhasil mengimpor {count} item makanan." });
            }
            catch (Exception ex)
            {
                return BadRequest(new { Error = ex.Message });
            }
        }
    }
}
