// Contoh: tulis laporan sampel PDF/DOCX/RTF ke /tmp untuk verifikasi manual.
use nutrisurvey_lib::export::{self, ExportFormat, ExportRequest, FoodEntry, MealTime, NutritionTargets};
use std::collections::HashMap;

fn main() {
    let request = ExportRequest {
        foods: vec![
            FoodEntry {
                meal_time: "SARAPAN".into(),
                name: "Nasi Putih".into(),
                amount: 200.0,
                serving_size: 100.0,
                serving_unit: Some("g".into()),
                nutrients: HashMap::from([
                    ("Energi".into(), 129.0),
                    ("karbohidrat total".into(), 28.2),
                    ("Protein".into(), 2.7),
                    ("Lemak total".into(), 0.3),
                    ("Kalsium".into(), 15.0),
                ]),
            },
            FoodEntry {
                meal_time: "SARAPAN".into(),
                name: "Telur Rebus".into(),
                amount: 50.0,
                serving_size: 100.0,
                serving_unit: Some("g".into()),
                nutrients: HashMap::from([
                    ("Energi (kkal)".into(), 154.0),
                    ("Energi".into(), 999.0),
                    ("karbohidrat total".into(), 1.1),
                    ("Protein".into(), 12.6),
                    ("Lemak total".into(), 10.6),
                ]),
            },
        ],
        meal_times: Some(vec![MealTime { id: "SARAPAN".into(), label: "Sarapan".into() }]),
        targets: Some(NutritionTargets { kcal: 2000.0, carbs: 300.0, protein: 50.0, fat: 70.0 }),
    };
    let template = include_bytes!("../../Assets/template.rtf");
    for (kind, bytes) in [
        (ExportFormat::Pdf, export::render_pdf(request.clone()).unwrap()),
        (ExportFormat::Docx, export::render_docx(request.clone()).unwrap()),
        (ExportFormat::Rtf, export::render_rtf(request, template).unwrap()),
    ] {
        let path = format!("/tmp/sample_report.{}", kind.extension());
        std::fs::write(&path, &bytes).unwrap();
        println!("{} ({} bytes)", path, bytes.len());
    }
}
