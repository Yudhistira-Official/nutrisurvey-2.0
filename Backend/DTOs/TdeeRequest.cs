using System.ComponentModel.DataAnnotations;

namespace NutritionApp.DTOs
{
    public class TdeeRequest
    {
        [Range(1, 500, ErrorMessage = "Berat badan harus antara 1 dan 500 kg")]
        public double WeightKg { get; set; }

        [Range(1, 300, ErrorMessage = "Tinggi badan harus antara 1 dan 300 cm")]
        public double HeightCm { get; set; }

        [Range(1, 150, ErrorMessage = "Usia harus antara 1 dan 150 tahun")]
        public int Age { get; set; }

        [Required(ErrorMessage = "Jenis kelamin wajib diisi")]
        public string Gender { get; set; } = "Male";

        public double ActivityFactor { get; set; } = 1.2;
        public double InjuryFactor { get; set; } = 1.0; // Default factor
        public bool IsManualFactors { get; set; } = false;
    }
}
