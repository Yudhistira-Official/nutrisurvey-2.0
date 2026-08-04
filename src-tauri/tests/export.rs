use nutrisurvey_lib::{
    commands,
    error::AppError,
    export::{self, ExportRequest, FoodEntry, MealTime, NutritionTargets},
};
use std::collections::HashMap;
use std::path::Path;

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
    assert!(output.contains("protein\\tab 0,0 g(0%)\\tab 50,0 g"));
    assert!(output.contains("energy\\tab 200,0 kcal\\tab 2000,0 kcal\\tab 10 %"));
    assert!(output.contains("carbohydr.\\tab 40,0 g(80%)\\tab 300,0 g\\tab 13 %"));
    assert!(output.contains("\\*\\themedata"));
}

#[test]
fn unicode_and_rtf_controls_are_escaped() {
    let mut item = food("Nasi {\u{1F35A}} \\");
    item.name.push_str("\nBaris\tDua");
    let bytes = export::render_rtf(request(vec![item]), b"prefix ===================================================================== suffix \\par }{\\*\\themedata").unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\\u"));
    assert!(output.contains("Nasi \\{"));
    assert!(output.contains("\\line Baris\\tab Dua"));
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
fn desktop_save_rejects_cancelled_dialog() {
    let error = export::validate_selected_path(None).unwrap_err();
    assert!(matches!(error, AppError::Export(message) if message == "Penyimpanan dibatalkan"));
}

#[test]
fn desktop_save_rejects_non_rtf_selected_path() {
    let error = export::validate_selected_path(Some(Path::new("report.docx"))).unwrap_err();
    assert!(matches!(error, AppError::Export(message) if message.contains(".rtf")));
}

#[test]
fn desktop_save_preserves_selected_path_conversion_error() {
    let selected: Option<Result<std::path::PathBuf, &str>> = Some(Err("invalid dialog path"));
    let error = export::resolve_selected_path(selected).unwrap_err();
    assert!(matches!(error, AppError::Export(message) if message == "invalid dialog path"));
}

#[test]
fn mobile_delivery_is_explicitly_unsupported_without_share_plugin() {
    let error = export::mobile_delivery_error().unwrap_err();
    assert!(
        matches!(error, AppError::Export(message) if message.contains("mobile") && message.contains("share"))
    );
}

#[test]
fn command_delivery_boundary_preserves_injected_cancel_error() {
    let result =
        commands::export_word_with_handler(request(Vec::new()), |_result: export::ExportResult| {
            Err(AppError::Export("Penyimpanan dibatalkan".into()))
        });
    assert!(
        matches!(result, Err(AppError::Export(message)) if message == "Penyimpanan dibatalkan")
    );
}

#[test]
fn command_delivery_boundary_preserves_injected_conversion_error() {
    let result =
        commands::export_word_with_handler(request(Vec::new()), |_result: export::ExportResult| {
            Err(AppError::Export("invalid dialog path".into()))
        });
    assert!(matches!(result, Err(AppError::Export(message)) if message == "invalid dialog path"));
}

#[test]
fn command_delivery_boundary_returns_injected_success() {
    let result = commands::export_word_with_handler(
        request(Vec::new()),
        |mut rendered: export::ExportResult| {
            rendered.saved_path = Some("selected/report.rtf".into());
            rendered.delivery = export::ExportDelivery::Saved;
            Ok(rendered)
        },
    )
    .unwrap();
    assert_eq!(result.delivery, export::ExportDelivery::Saved);
    assert_eq!(result.saved_path.as_deref(), Some("selected/report.rtf"));
}

#[test]
fn command_delivery_boundary_preserves_invalid_extension_error() {
    let result =
        commands::export_word_with_handler(request(Vec::new()), |_result: export::ExportResult| {
            Err(AppError::Export(
                "Lokasi penyimpanan harus berakhiran .rtf".into(),
            ))
        });
    assert!(matches!(result, Err(AppError::Export(message)) if message.contains(".rtf")));
}

#[test]
fn command_delivery_boundary_preserves_mobile_unsupported_error() {
    let result =
        commands::export_word_with_handler(request(Vec::new()), |_result: export::ExportResult| {
            export::mobile_delivery_error()
        });
    assert!(matches!(result, Err(AppError::Export(message)) if message.contains("mobile")));
}

#[test]
fn result_has_rtf_content_type_and_date_filename() {
    let result = export::result(vec![1, 2, 3]);
    assert_eq!(result.content_type, "application/rtf");
    assert!(result.filename.starts_with("Laporan_Nutrisi_"));
    assert!(result.filename.ends_with(".rtf"));
    assert_eq!(result.bytes, vec![1, 2, 3]);
    assert_eq!(result.delivery, export::ExportDelivery::Share);
    assert!(result.saved_path.is_none());
}
