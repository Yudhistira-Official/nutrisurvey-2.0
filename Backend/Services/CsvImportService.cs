using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using CsvHelper;
using CsvHelper.Configuration;
using Microsoft.AspNetCore.Http;
using Microsoft.EntityFrameworkCore;
using NutritionApp.Data;
using NutritionApp.Models;

namespace NutritionApp.Services
{
    public interface ICsvImportService
    {
        Task<int> ImportFoodDatabaseAsync(IFormFile file);
        Task<int> ImportFromFileAsync(string filePath);
    }

    public class CsvImportService : ICsvImportService
    {
        private readonly AppDbContext _context;

        public CsvImportService(AppDbContext context)
        {
            _context = context;
        }

        public async Task<int> ImportFoodDatabaseAsync(IFormFile file)
        {
            if (file == null || file.Length == 0) return 0;

            var tempPath = Path.GetTempFileName();
            using (var stream = new FileStream(tempPath, FileMode.Create))
            {
                await file.CopyToAsync(stream);
            }

            try
            {
                return await ImportFromFileAsync(tempPath);
            }
            finally
            {
                if (File.Exists(tempPath)) File.Delete(tempPath);
            }
        }

        public async Task<int> ImportFromFileAsync(string filePath)
        {
            if (!File.Exists(filePath)) return 0;

            var delimiter = DetectDelimiter(filePath);
            using var reader = new StreamReader(filePath, System.Text.Encoding.UTF8);
            var config = new CsvConfiguration(CultureInfo.InvariantCulture)
            {
                HasHeaderRecord = true,
                MissingFieldFound = null,
                HeaderValidated = null,
                Delimiter = delimiter
            };
            using var csv = new CsvReader(reader, config);

            if (!csv.Read() || !csv.ReadHeader()) return 0;
            var headers = csv.HeaderRecord;
            if (headers == null) throw new Exception("CSV headers could not be read.");

            // Detect format
            bool isScraperFormat = headers.Contains("makanan") && headers.Any(h => h.StartsWith("komponen_nutrient_"));

            if (isScraperFormat)
                return await ImportScraperFormatAsync(csv, headers);
            else
                return await ImportStandardFormatAsync(csv, headers);
        }

        private async Task<int> ImportStandardFormatAsync(CsvReader csv, string[] headers)
        {
            int foodNameIndex = Array.IndexOf(headers, "Nama Makanan");
            if (foodNameIndex == -1) foodNameIndex = Array.IndexOf(headers, "FoodName");
            
            int categoryIndex = Array.IndexOf(headers, "Kategori");
            if (categoryIndex == -1) categoryIndex = Array.IndexOf(headers, "Category");

            int servPerContIndex = Array.IndexOf(headers, "Jumlah Sajian");
            int perSajianIndex = Array.IndexOf(headers, "Per Sajian");

            if (foodNameIndex == -1) throw new Exception("Header 'Nama Makanan' tidak ditemukan.");

            int importedCount = 0;
            _context.ChangeTracker.AutoDetectChangesEnabled = false;
            var allNutrients = await _context.Nutrients.ToListAsync();
            var nutrientDict = allNutrients.ToDictionary(n => n.Name.ToLower(), n => n);

            while (csv.Read())
            {
                string? foodName = csv.GetField(foodNameIndex);
                if (string.IsNullOrWhiteSpace(foodName)) continue;

                string? category = categoryIndex != -1 ? csv.GetField(categoryIndex) : null;
                
                var food = new Food { Name = foodName, Category = category };
                
                // Set serving info
                if (servPerContIndex != -1)
                {
                    var spcVal = csv.GetField(servPerContIndex);
                    if (double.TryParse(spcVal?.Replace(',', '.'), NumberStyles.Any, CultureInfo.InvariantCulture, out double spc))
                        food.ServingsPerContainer = spc;
                }

                if (perSajianIndex != -1)
                {
                    var perSajianVal = csv.GetField(perSajianIndex);
                    var (size, unit) = ParseServingInfo(perSajianVal);
                    if (size.HasValue) food.ServingSize = size.Value;
                    if (!string.IsNullOrEmpty(unit)) food.ServingUnit = unit;
                }

                _context.Foods.Add(food);
                await _context.SaveChangesAsync(); // Need ID for FoodNutrients

                for (int i = 0; i < headers.Length; i++)
                {
                    string? headerName = headers[i];
                    if (string.IsNullOrWhiteSpace(headerName) || IsMetadataColumn(headerName)) continue;
                    if (i == foodNameIndex || i == categoryIndex) continue;

                    string? cellValue = csv.GetField(i);
                    if (string.IsNullOrWhiteSpace(cellValue) || cellValue == "0" || cellValue == "-") continue;

                    var (amount, unit) = ParseNutrientValue(cellValue);
                    if (amount == null) continue;

                    string nKey = headerName.ToLower();
                    if (!nutrientDict.TryGetValue(nKey, out var nutrient))
                    {
                        nutrient = new Nutrient { Name = headerName, Unit = unit ?? "mg" };
                        _context.Nutrients.Add(nutrient);
                        await _context.SaveChangesAsync();
                        nutrientDict[nKey] = nutrient;
                    }

                    _context.FoodNutrients.Add(new FoodNutrient { 
                        FoodId = food.Id, 
                        NutrientId = nutrient.Id, 
                        Amount = amount.Value 
                    });
                }
                
                importedCount++;
                if (importedCount % 50 == 0) 
                {
                    await _context.SaveChangesAsync();
                    Console.WriteLine($"Imported {importedCount} foods...");
                }
            }
            await _context.SaveChangesAsync();
            _context.ChangeTracker.AutoDetectChangesEnabled = true;
            return importedCount;
        }

        private async Task<int> ImportScraperFormatAsync(CsvReader csv, string[] headers)
        {
            int importedCount = 0;
            var allNutrients = await _context.Nutrients.ToListAsync();

            while (csv.Read())
            {
                string? foodName = csv.GetField("makanan");
                if (string.IsNullOrWhiteSpace(foodName)) continue;

                // Clean food name from scraper metadata (e.g. Kis Mint Cherry 125g;PT. Mayora...)
                foodName = foodName.Split(';')[0].Trim();

                string? category = csv.GetField("kategori");
                var food = await GetOrCreateFoodAsync(foodName, category);

                // Scraper format has komponen_nutrient_1 paired with isi_nutrient_1
                for (int i = 1; i <= 48; i++)
                {
                    string kompHeader = $"komponen_nutrient_{i}";
                    string isiHeader = $"isi_nutrient_{i}";

                    if (!headers.Contains(kompHeader) || !headers.Contains(isiHeader)) continue;

                    string? nutrientName = csv.GetField(kompHeader);
                    string? nutrientValue = csv.GetField(isiHeader);

                    if (string.IsNullOrWhiteSpace(nutrientName) || string.IsNullOrWhiteSpace(nutrientValue)) continue;

                    await ProcessNutrientAsync(food, nutrientName, nutrientValue, allNutrients);
                }

                importedCount++;
                if (importedCount % 50 == 0) await _context.SaveChangesAsync();
            }
            await _context.SaveChangesAsync();
            return importedCount;
        }

        private async Task<Food> GetOrCreateFoodAsync(string name, string? category)
        {
            var food = await _context.Foods.Include(f => f.FoodNutrients)
                        .FirstOrDefaultAsync(f => f.Name == name);
            
            if (food == null) {
                food = new Food { Name = name, Category = category };
                _context.Foods.Add(food);
                await _context.SaveChangesAsync();
            }
            else if (string.IsNullOrEmpty(food.Category) && !string.IsNullOrEmpty(category)) {
                food.Category = category;
            }
            return food;
        }

        private async Task ProcessNutrientAsync(Food food, string nutrientName, string? value, List<Nutrient> allNutrients)
        {
            if (string.IsNullOrWhiteSpace(value) || value == "-") return;

            var (amount, unit) = ParseNutrientValue(value);
            if (amount == null) return;

            var nutrient = allNutrients.FirstOrDefault(n => n.Name.Equals(nutrientName, StringComparison.OrdinalIgnoreCase));
            if (nutrient == null)
            {
                nutrient = new Nutrient { Name = nutrientName, Unit = unit ?? "mg" };
                _context.Nutrients.Add(nutrient);
                await _context.SaveChangesAsync();
                allNutrients.Add(nutrient);
            }

            var foodNutrient = food.FoodNutrients.FirstOrDefault(fn => fn.NutrientId == nutrient.Id);
            if (foodNutrient == null) {
                _context.FoodNutrients.Add(new FoodNutrient { 
                    FoodId = food.Id, 
                    NutrientId = nutrient.Id, 
                    Amount = amount.Value 
                });
            } else {
                foodNutrient.Amount = amount.Value; // Overwrite/Update
            }
        }

        private bool IsMetadataColumn(string header)
        {
            string h = header.ToLower();
            return h == "id" || h == "jumlah sajian" || h == "per sajian" || h.Contains("web_scraper");
        }

        private string DetectDelimiter(string filePath)
        {
            using var reader = new StreamReader(filePath, System.Text.Encoding.UTF8);
            var firstLine = reader.ReadLine();
            if (string.IsNullOrWhiteSpace(firstLine)) return ";";

            int semicolonCount = firstLine.Count(c => c == ';');
            int commaCount = firstLine.Count(c => c == ',');
            return semicolonCount >= commaCount ? ";" : ",";
        }

        private (double? Amount, string? Unit) ParseServingInfo(string? value)
        {
            if (string.IsNullOrWhiteSpace(value)) return (null, null);

            var normalized = value.Trim().Replace(',', '.');
            var direct = ParseNutrientValue(normalized);
            if (direct.Amount.HasValue) return direct;

            var match = Regex.Match(normalized, @"\(?\s*([0-9]+(?:\.[0-9]+)?)\s*([a-zA-Z]+)\s*\)?");
            if (!match.Success) return (null, null);

            if (!double.TryParse(match.Groups[1].Value, NumberStyles.Any, CultureInfo.InvariantCulture, out double result))
            {
                return (null, null);
            }

            var unit = match.Groups[2].Value;
            return (result, string.IsNullOrWhiteSpace(unit) ? null : unit);
        }

        private (double? Amount, string? Unit) ParseNutrientValue(string value)
        {
            if (string.IsNullOrWhiteSpace(value)) return (null, null);

            // Regex to match decimal number and capture optional unit after space or directly
            // e.g. "364 kkal", "3.50 g", "0.33mg", "1,5"
            var match = Regex.Match(value.Trim().Replace(',', '.'), @"^([0-9\.]+)\s*([a-zA-Z]*)$");
            
            if (!match.Success) return (null, null);

            string numberStr = match.Groups[1].Value;
            string unit = match.Groups[2].Value;

            if (double.TryParse(numberStr, NumberStyles.Any, CultureInfo.InvariantCulture, out double result))
            {
                return (result, string.IsNullOrEmpty(unit) ? null : unit);
            }

            return (null, null);
        }
    }
}
