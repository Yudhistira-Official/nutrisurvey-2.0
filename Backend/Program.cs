using Microsoft.EntityFrameworkCore;
using NutritionApp.Data;
using NutritionApp.Services;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
builder.Services.AddControllers();

// Configure Database (SQLite)
builder.Services.AddDbContext<AppDbContext>(options =>
    options.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection") ?? "Data Source=nutrition.db"));

// Register Services for Dependency Injection
builder.Services.AddScoped<ICsvImportService, CsvImportService>();
builder.Services.AddScoped<INutritionCalculatorService, NutritionCalculatorService>();

// Configure CORS for Frontend (allow access from port 8080)
builder.Services.AddCors(options =>
{
    options.AddPolicy("AllowFrontend", policy =>
    {
        policy.WithOrigins("http://localhost:8080")
              .AllowAnyHeader()
              .AllowAnyMethod();
    });
});

var app = builder.Build();

// Ensure the database and tables are created automatically on startup
using (var scope = app.Services.CreateScope())
{
    var services = scope.ServiceProvider;
    var dbContext = services.GetRequiredService<AppDbContext>();
    dbContext.Database.EnsureCreated();

    // Auto-seed from DatabaseMakanan/nilaigizi_clean.csv if empty
    var foodCount = dbContext.Foods.Count();
    Console.WriteLine($"Current food count in database: {foodCount}");

    if (foodCount == 0)
    {
        var importService = services.GetRequiredService<ICsvImportService>();
        
        string directoryPath = Path.Combine(Directory.GetCurrentDirectory(), "DatabaseMakanan");
        if (!Directory.Exists(directoryPath))
        {
            directoryPath = Path.GetFullPath(Path.Combine(Directory.GetCurrentDirectory(), "..", "DatabaseMakanan"));
        }

        if (Directory.Exists(directoryPath))
        {
            var csvFiles = Directory.GetFiles(directoryPath, "*.csv", SearchOption.AllDirectories);
            foreach (var csvPath in csvFiles)
            {
                Console.WriteLine($"Importing from {csvPath}...");
                try 
                {
                    var count = importService.ImportFromFileAsync(csvPath).GetAwaiter().GetResult();
                    Console.WriteLine($"Imported {count} items from {Path.GetFileName(csvPath)}.");
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"Error importing {Path.GetFileName(csvPath)}: {ex.Message}");
                }
            }
        }
    }
}

app.UseCors("AllowFrontend");
app.UseAuthorization();
app.MapControllers();

app.Run();