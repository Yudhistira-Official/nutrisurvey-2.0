using Microsoft.AspNetCore.Mvc;
using NutritionApp.DTOs;
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;

namespace NutritionApp.Controllers
{
    [ApiController]
    [Route("api/[controller]")]
    public class ExportController : ControllerBase
    {
        [HttpPost("word")]
        public IActionResult ExportToWord([FromBody] ExportRequest request)
        {
            if (request == null || request.Foods == null) return BadRequest("Data kosong");
            var grandTotals = new Dictionary<string, double>();
            var mealTimes = request.MealTimes ?? new List<MealTimeDto> {
                new MealTimeDto { Id = "SARAPAN", Label = "Sarapan" },
                new MealTimeDto { Id = "MAKAN_SIANG", Label = "Makan Siang" },
                new MealTimeDto { Id = "MAKAN_MALAM", Label = "Makan Malam" }
            };

            var mealRows = new List<(string Label, List<(string Name, string AmountText, double Energy, double Carbs)> Foods, double Energy, double Carbs)>();

            foreach (var meal in mealTimes)
            {
                var foods = request.Foods.Where(f => f.MealTime == meal.Id).ToList();
                var foodRows = new List<(string Name, string AmountText, double Energy, double Carbs)>();
                double mealEnergy = 0;
                double mealCarbs = 0;

                foreach (var f in foods)
                {
                    var baseSize = f.ServingSize > 0 ? f.ServingSize : 100;
                    double ratio = (f.Amount > 0 ? f.Amount : 0) / baseSize;
                    double energy = (GetByAliases(f.Nutrients, "energy", "energi") * ratio);
                    double carbs = (GetByAliases(f.Nutrients, "carbohydrate", "carbohydr.", "karbohidrat", "karbohidrat total", "carbs", "karbo") * ratio);

                    mealEnergy += energy;
                    mealCarbs += carbs;

                    var unit = string.IsNullOrWhiteSpace(f.ServingUnit) ? "g" : f.ServingUnit;
                    foodRows.Add((f.Name, $"{f.Amount:0.#} {unit}", energy, carbs));

                    foreach (var n in f.Nutrients)
                    {
                        var key = n.Key?.Trim().ToLowerInvariant();
                        if (string.IsNullOrWhiteSpace(key)) continue;
                        double val = (n.Value) * ratio;
                        grandTotals[key] = grandTotals.GetValueOrDefault(key, 0) + val;
                    }
                }

                mealRows.Add((meal.Label, foodRows, mealEnergy, mealCarbs));
            }

            var totalEnergy = mealRows.Sum(m => m.Energy);
            var totalCarbs = mealRows.Sum(m => m.Carbs);

            var sb = new StringBuilder();
            sb.Append("=====================================================================\\par ");
            sb.Append("\\pard \\ltrpar\\qc \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 ");
            sb.Append("{\\rtlch\\fcs1 \\ab\\af0\\afs30 \\ltrch\\fcs0 \\b\\f0\\fs30\\kerning0 ");
            sb.Append("HASIL PERHITUNGAN DIET/\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 ");
            sb.Append("{\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 ");
            sb.Append("=====================================================================\\par }");
            sb.Append("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqc\\tx5500\\tqc\\tx7100\\tqc\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 ");
            sb.Append("{\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 Nama Makanan\\tab Jumlah\\tab energy\\tab carbohydr.\\par }");
            sb.Append("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 ");
            sb.Append("{\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 ______________________________________________________________________________ \\par }");
            sb.Append("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqdec\\tx5500\\tqdec\\tx7100\\tqdec\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 ");
            sb.Append("{\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\par }");

            foreach (var meal in mealRows)
            {
                sb.Append("{\\rtlch\\fcs1 \\ab\\af0 \\ltrch\\fcs0 \\b\\f0\\kerning0 ");
                sb.Append(EscapeRtfText(meal.Label.ToUpperInvariant()));
                sb.Append("}{\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\par ");
                foreach (var f in meal.Foods)
                {
                    sb.Append("\\hich\\af0\\dbch\\af31505\\loch\\f0 ");
                    sb.Append(EscapeRtfText(f.Name));
                    sb.Append("\\tab ");
                    sb.Append(EscapeRtfText(f.AmountText));
                    sb.Append("\\tab ");
                    sb.Append(FmtNum(f.Energy, 1, 8));
                    sb.Append(" kcal\\tab ");
                    sb.Append(FmtNum(f.Carbs, 1, 7));
                    sb.Append("  g\\par ");
                }
                var ePct = totalEnergy > 0 ? (meal.Energy / totalEnergy) * 100 : 0;
                var cPct = totalCarbs > 0 ? (meal.Carbs / totalCarbs) * 100 : 0;
                sb.Append("\\par \\hich\\af0\\dbch\\af31505\\loch\\f0 Meal analysis:  energy ");
                sb.Append(FmtNum(meal.Energy, 1, 0));
                sb.Append(" kcal (");
                sb.Append(ePct.ToString("0"));
                sb.Append(" %),  carbohydrate ");
                sb.Append(FmtNum(meal.Carbs, 1, 0));
                sb.Append(" g (");
                sb.Append(cPct.ToString("0"));
                sb.Append(" %)\\par \\par \\par ");
                sb.Append("}");
            }

            sb.Append("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\par \\hich\\af0\\dbch\\af31505\\loch\\f0 =====================================================================\\par }");
            sb.Append("\\pard \\ltrpar\\qc \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\ab\\af0\\afs30 \\ltrch\\fcs0 \\b\\f0\\fs30\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 HASIL PERHITUNGAN\\par }");
            sb.Append("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 =====================================================================\\par }");
            sb.Append("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqc\\tx2900\\tqc\\tx5600\\tqc\\tx8300\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 Zat Gizi\\tab hasil analisis\\tab rekomendasi\\tab persentase\\par \\hich\\af0\\dbch\\af31505\\loch\\f0      \\tab nilai\\tab nilai/hari\\tab pemenuhan\\par }");
            sb.Append("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 ______________________________________________________________________________\\par }");
            sb.Append("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqdec\\tx3000\\tqdec\\tx5700\\tqdec\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 ");

            var nutrientRows = BuildNutrientRows(grandTotals, request.Targets, totalEnergy);
            foreach (var row in nutrientRows)
            {
                sb.Append(EscapeRtfText(row.Name));
                sb.Append("\\tab ");
                sb.Append(ToTemplateUnits(EscapeRtfText(row.Analysis)));
                sb.Append("\\tab ");
                sb.Append(ToTemplateUnits(EscapeRtfText(row.Recommendation)));
                sb.Append("\\tab ");
                sb.Append(EscapeRtfText(row.Percent));
                sb.Append(" \\par ");
            }
            sb.Append("}");

            var templatePath = Path.Combine(Directory.GetCurrentDirectory(), "..", "Assets", "template.rtf");
            var templateRaw = System.IO.File.ReadAllText(templatePath, Encoding.ASCII);
            var startMarker = "=====================================================================";
            var endMarker = "\\par }{\\*\\themedata";
            var startIdx = templateRaw.IndexOf(startMarker, StringComparison.Ordinal);
            var endIdx = templateRaw.IndexOf(endMarker, StringComparison.Ordinal);
            if (startIdx < 0 || endIdx <= startIdx) return BadRequest("Template RTF tidak valid");

            var finalRtf = templateRaw.Substring(0, startIdx) + sb + "\\par }" + templateRaw.Substring(endIdx + "\\par }".Length);
            return File(Encoding.ASCII.GetBytes(finalRtf), "application/rtf", $"Laporan_Nutrisi_{DateTime.Now:yyyyMMdd}.rtf");
        }

        private static string EscapeRtfText(string value)
        {
            if (string.IsNullOrEmpty(value)) return string.Empty;
            return value
                .Replace("\\", "\\\\")
                .Replace("{", "\\{")
                .Replace("}", "\\}");
        }

        private static double GetByAliases(Dictionary<string, double> nutrients, params string[] aliases)
        {
            if (nutrients == null || nutrients.Count == 0) return 0;

            var normalizedAliases = aliases
                .Where(a => !string.IsNullOrWhiteSpace(a))
                .Select(NormalizeNutrientKey)
                .ToList();

            double exact = 0;
            bool hasExact = false;

            foreach (var kv in nutrients)
            {
                if (string.IsNullOrWhiteSpace(kv.Key)) continue;
                var key = NormalizeNutrientKey(kv.Key);
                if (normalizedAliases.Contains(key))
                {
                    exact += kv.Value;
                    hasExact = true;
                }
            }

            if (hasExact) return exact;

            double partial = 0;
            foreach (var kv in nutrients)
            {
                if (string.IsNullOrWhiteSpace(kv.Key)) continue;
                var key = NormalizeNutrientKey(kv.Key);
                if (normalizedAliases.Any(alias => key.Contains(alias) || alias.Contains(key)))
                {
                    partial += kv.Value;
                }
            }

            return partial;
        }

        private static string NormalizeNutrientKey(string value)
        {
            var chars = value
                .Trim()
                .ToLowerInvariant()
                .Select(ch => char.IsLetterOrDigit(ch) ? ch : ' ')
                .ToArray();

            return string.Join(" ", new string(chars).Split(' ', StringSplitOptions.RemoveEmptyEntries));
        }

        private static List<(string Name, string Analysis, string Recommendation, string Percent)> BuildNutrientRows(
            Dictionary<string, double> totals,
            NutritionTargetsDto? targets,
            double totalEnergy)
        {
            var map = new List<(string Label, string Unit, double Recommendation, string[] Aliases)>
            {
                ("energy", "kcal", targets?.Kcal > 0 ? targets.Kcal : 1900, new[] { "energy", "energi" }),
                ("water", "g", 2600, new[] { "water", "air" }),
                ("protein", "g", targets?.Protein > 0 ? targets.Protein : 47, new[] { "protein" }),
                ("fat", "g", targets?.Fat > 0 ? targets.Fat : 73, new[] { "fat", "lemak" }),
                ("carbohydr.", "g", targets?.Carbs > 0 ? targets.Carbs : 332, new[] { "carbohydrate", "carbohydr.", "karbohidrat", "karbohidrat total", "carbs", "karbo" }),
                ("dietary fiber", "g", 30, new[] { "dietary fiber", "fiber", "serat" }),
                ("alcohol", "g", 0, new[] { "alcohol" }),
                ("PUFA", "g", 10, new[] { "pufa" }),
                ("cholesterol", "mg", 0, new[] { "cholesterol" }),
                ("Vit. A", "ug", 800, new[] { "vit. a", "vitamin a" }),
                ("carotene", "mg", 0, new[] { "carotene" }),
                ("Vit. E", "mg", 0, new[] { "vit. e", "vitamin e" }),
                ("Vit. B1", "mg", 1, new[] { "vit. b1", "vitamin b1", "thiamin" }),
                ("Vit. B2", "mg", 1.2, new[] { "vit. b2", "vitamin b2", "riboflavin" }),
                ("Vit. B6", "mg", 1.2, new[] { "vit. b6", "vitamin b6" }),
                ("folic acid eq.", "ug", 0, new[] { "folic acid eq.", "folate" }),
                ("Vit. C", "mg", 100, new[] { "vit. c", "vitamin c" }),
                ("sodium", "mg", 2000, new[] { "sodium", "natrium" }),
                ("potassium", "mg", 3500, new[] { "potassium", "kalium" }),
                ("calcium", "mg", 1000, new[] { "calcium", "kalsium" }),
                ("magnesium", "mg", 300, new[] { "magnesium" }),
                ("phosphorus", "mg", 700, new[] { "phosphorus", "fosfor" }),
                ("iron", "mg", 15, new[] { "iron", "zat besi" }),
                ("zinc", "mg", 7, new[] { "zinc", "seng" })
            };

            var rows = new List<(string Name, string Analysis, string Recommendation, string Percent)>();
            foreach (var item in map)
            {
                double value = GetByAliases(totals, item.Aliases);

                string analysis = $"{FmtNum(value, 1, 0)} {item.Unit}";
                string rec = item.Recommendation > 0 ? $"{FmtNum(item.Recommendation, 1, 0)} {item.Unit}" : "-";
                string pct = item.Recommendation > 0 ? $"{Math.Round((value / item.Recommendation) * 100):0} %" : "-";

                if (item.Label == "protein")
                {
                    var macroPct = totalEnergy > 0 ? ((value * 4) / totalEnergy) * 100 : 0;
                    analysis = $"{FmtNum(value, 1, 0)} {item.Unit}({macroPct:0}%)";
                }
                if (item.Label == "fat")
                {
                    var macroPct = totalEnergy > 0 ? ((value * 9) / totalEnergy) * 100 : 0;
                    analysis = $"{FmtNum(value, 1, 0)} {item.Unit}({macroPct:0}%)";
                }
                if (item.Label == "carbohydr.")
                {
                    var macroPct = totalEnergy > 0 ? ((value * 4) / totalEnergy) * 100 : 0;
                    analysis = $"{FmtNum(value, 1, 0)} {item.Unit}({macroPct:0}%)";
                }

                rows.Add((item.Label, analysis, rec, pct));
            }

            return rows;
        }

        private static string FmtNum(double value, int decimals, int width)
        {
            var s = value.ToString($"F{decimals}", CultureInfo.InvariantCulture).Replace('.', ',');
            return width > 0 ? s.PadLeft(width) : s;
        }

        private static string ToTemplateUnits(string text)
        {
            return text.Replace(" ug", " \\hich\\f0 \'b5\\loch\\f0 g");
        }
    }
}
