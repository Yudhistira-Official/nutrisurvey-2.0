use nutrisurvey_lib::export::{self, ExportRequest, FoodEntry, MealTime, NutritionTargets};
use std::collections::HashMap;

fn request(foods: Vec<FoodEntry>) -> ExportRequest {
    ExportRequest {
        foods,
        meal_times: Some(vec![MealTime {
            id: "SARAPAN".into(),
            label: "Sarapan".into(),
        }]),
        targets: Some(NutritionTargets {
            kcal: 2000.0,
            carbs: 300.0,
            protein: 50.0,
            fat: 70.0,
        }),
    }
}

fn food(name: &str) -> FoodEntry {
    FoodEntry {
        meal_time: "SARAPAN".into(),
        name: name.into(),
        amount: 100.0,
        serving_size: 100.0,
        serving_unit: Some("g".into()),
        nutrients: HashMap::from([("energy".into(), 100.0), ("carbohydrate".into(), 20.0)]),
    }
}

#[test]
fn bundled_template_renders_expected_report_sections() {
    let bytes = export::render_rtf(
        request(vec![food("Nasi"), food("Telur")]),
        include_bytes!("../../Assets/template.rtf"),
    )
    .unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("HASIL PERHITUNGAN DIET"));
    assert!(output.contains("Nasi"));
    assert!(output.contains("Telur"));
    assert!(output.contains("energy"));
    assert!(output.contains("2000,0"));
}

#[test]
fn unicode_and_rtf_controls_are_escaped() {
    let bytes = export::render_rtf(request(vec![food("Nasi {\u{1F35A}} \\" )]), b"prefix ===================================================================== suffix \\par }{\\*\\themedata").unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\\u"));
    assert!(output.contains("Nasi \\{"));
    assert!(!output.contains("Nasi {"));
}

#[test]
fn empty_meals_render_zero_analysis_without_error() {
    let bytes = export::render_rtf(
        request(Vec::new()),
        include_bytes!("../../Assets/template.rtf"),
    )
    .unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("SARAPAN"));
    assert!(output.contains("0,0 kcal (0 %)"));
}

#[test]
fn invalid_template_returns_export_error() {
    let error = export::render_rtf(request(Vec::new()), b"not rtf").unwrap_err();
    assert!(matches!(error, nutrisurvey_lib::error::AppError::Export(_)));
}

#[test]
fn result_has_rtf_content_type_and_date_filename() {
    let result = export::result(vec![1, 2, 3]);
    assert_eq!(result.content_type, "application/rtf");
    assert!(result.filename.starts_with("Laporan_Nutrisi_"));
    assert!(result.filename.ends_with(".rtf"));
    assert_eq!(result.bytes, vec![1, 2, 3]);
    assert!(result.saved_path.is_none());
}
