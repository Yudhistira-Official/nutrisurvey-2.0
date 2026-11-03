use nutrisurvey_lib::{
    commands,
    error::AppError,
    export::{self, ExportFormat, ExportRequest, FoodEntry, MealTime, NutritionTargets},
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
fn frontend_camel_case_payload_keeps_meals_and_targets() {
    // Payload persis seperti yang dikirim frontend (camelCase).
    let json = r#"{
        "foods": [{
            "mealTime": "BREAKFAST",
            "name": "Nasi Putih",
            "amount": 200.0,
            "servingSize": 100.0,
            "servingUnit": "g",
            "nutrients": {"energi": 129.0, "karbohidrat total": 28.2}
        }],
        "mealTimes": [{"id": "BREAKFAST", "label": "Makan Pagi"}],
        "targets": {"kcal": 2035.0, "carbs": 254.0, "protein": 102.0, "fat": 68.0}
    }"#;
    let request: ExportRequest = serde_json::from_str(json).unwrap();
    assert!(request.meal_times.is_some(), "mealTimes terbuang saat deserialisasi");
    assert!(request.targets.is_some(), "targets terbuang saat deserialisasi");
    let bytes = export::render_rtf(request, include_bytes!("../../Assets/template.rtf")).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("Makan Pagi"), "label waktu makan dari project hilang");
    assert!(output.contains("Nasi Putih"), "baris makanan kosong");
    assert!(output.contains("258,0 kcal"), "energi makanan tidak dihitung");
    assert!(output.contains("2035,0 kcal"), "target project tidak dipakai");
}

#[test]
fn meal_times_missing_are_derived_from_foods_instead_of_empty_report() {
    let json = r#"{
        "foods": [{
            "mealTime": "SNACK",
            "name": "Apel",
            "amount": 100.0,
            "servingSize": 100.0,
            "servingUnit": "g",
            "nutrients": {"energi": 52.0}
        }]
    }"#;
    let request: ExportRequest = serde_json::from_str(json).unwrap();
    let bytes = export::render_rtf(request, include_bytes!("../../Assets/template.rtf")).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("SNACK"));
    assert!(output.contains("Apel"));
    assert!(output.contains("52,0 kcal"));
}

#[test]
fn rendered_rtf_is_brace_balanced_like_template() {
    let bytes = export::render_rtf(
        request(vec![food("Nasi"), food("Telur")]),
        include_bytes!("../../Assets/template.rtf"),
    )
    .unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let raw = text.as_bytes();
    let mut depth: i64 = 0;
    let mut index = 0;
    while index < raw.len() {
        match raw[index] {
            b'\\' => index += 1,
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        assert!(depth >= 0, "brace ditutup berlebihan pada byte {index}");
        index += 1;
    }
    assert_eq!(depth, 0, "RTF hasil render harus seimbang seperti template");
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
    assert!(!output.contains("HASIL PERHITUNGAN DIET/"));
    assert!(output.contains("\\tqc\\tx3000\\tqc\\tx5700\\tqc\\tx8400"));
    assert!(output.contains("Nasi"));
    assert!(output.contains("Telur"));
    assert!(output.contains("energy"));
    assert!(output.contains("2000,0"));
    assert!(output.contains("protein\\tab 0,0 g(0%)\\tab 50,0 g(10 %)"));
    assert!(output.contains("energy\\tab 200,0 kcal\\tab 2000,0 kcal\\tab 10 %"));
    assert!(output.contains("carbohydr.\\tab 40,0 g(80%)\\tab 300,0 g(> 55 %)\\tab 13 %"));
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
    assert!(output.contains("Sarapan"));
    assert!(output.contains("0,0 kcal (0 %)"));
}

#[test]
fn invalid_template_returns_export_error() {
    let error = export::render_rtf(request(Vec::new()), b"not rtf").unwrap_err();
    assert!(matches!(error, nutrisurvey_lib::error::AppError::Export(_)));
}

#[test]
fn desktop_save_rejects_cancelled_dialog() {
    let error = export::validate_selected_path(None, ExportFormat::Rtf).unwrap_err();
    assert!(matches!(error, AppError::Export(message) if message == "Penyimpanan dibatalkan"));
}

#[test]
fn desktop_save_rejects_non_rtf_selected_path() {
    let error = export::validate_selected_path(Some(Path::new("report.pdf")), ExportFormat::Rtf)
        .unwrap_err();
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
fn mobile_delivery_result_never_claims_share_without_delivery() {
    let result = export::mobile_delivery_result(vec![1, 2, 3]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("mobile"));
}

#[test]
fn result_has_rtf_content_type_and_date_filename() {
    let result = export::result(ExportFormat::Rtf, vec![1, 2, 3]);
    assert_eq!(result.content_type, "application/rtf");
    assert!(result.filename.starts_with("Laporan_Nutrisi_"));
    assert!(result.filename.ends_with(".rtf"));
    assert_eq!(result.bytes, vec![1, 2, 3]);
    assert_eq!(result.delivery, export::ExportDelivery::Share);
    assert!(result.saved_path.is_none());
}

#[test]
fn format_parses_pdf_docx_and_rtf_and_rejects_unknown() {
    assert_eq!(export::ExportFormat::parse("pdf").unwrap(), ExportFormat::Pdf);
    assert_eq!(export::ExportFormat::parse("docx").unwrap(), ExportFormat::Docx);
    assert_eq!(export::ExportFormat::parse("rtf").unwrap(), ExportFormat::Rtf);
    assert_eq!(export::ExportFormat::parse("PDF").unwrap(), ExportFormat::Pdf);
    assert!(export::ExportFormat::parse("xlsx").is_err());
    assert!(export::ExportFormat::parse("").is_err());
}

#[test]
fn result_uses_format_specific_filename_and_content_type() {
    let pdf = export::result(ExportFormat::Pdf, vec![]);
    assert!(pdf.filename.ends_with(".pdf"));
    assert_eq!(pdf.content_type, "application/pdf");
    let docx = export::result(ExportFormat::Docx, vec![]);
    assert!(docx.filename.ends_with(".docx"));
    assert_eq!(
        docx.content_type,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    );
}

#[test]
fn docx_output_is_valid_zip_containing_report_text() {
    let bytes = export::render_docx(request(vec![food("Nasi")])).unwrap();
    assert_eq!(&bytes[..2], b"PK");
    let (entries, _) = export::test_support::parse_stored_zip(&bytes);
    assert!(entries.iter().any(|(name, _)| name == "word/document.xml"));
    let document = entries
        .iter()
        .find(|(name, _)| name == "word/document.xml")
        .map(|(_, data)| String::from_utf8_lossy(data).to_string())
        .unwrap();
    assert!(document.contains("HASIL PERHITUNGAN DIET"));
    assert!(document.contains("Nasi"));
    assert!(document.contains("2000,0"));
}

#[test]
fn docx_meal_and_nutrient_tables_match_request() {
    let bytes = export::render_docx(request(vec![food("Nasi"), food("Telur")])).unwrap();
    let (entries, _) = export::test_support::parse_stored_zip(&bytes);
    let document = entries
        .iter()
        .find(|(name, _)| name == "word/document.xml")
        .map(|(_, data)| String::from_utf8_lossy(data).to_string())
        .unwrap();
    assert!(document.contains("Sarapan"));
    assert!(document.contains("Telur"));
    assert!(document.contains("energy"));
    assert!(document.contains("carbohydr."));
}

#[test]
fn pdf_output_contains_report_sections() {
    let bytes = export::render_pdf(request(vec![food("Nasi"), food("Telur")])).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    assert!(bytes.windows(5).any(|w| w == b"%%EOF"));
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("HASIL PERHITUNGAN DIET"));
    assert!(text.contains("Nasi"));
    assert!(text.contains("Telur"));
    assert!(text.contains("2000,0"));
}

#[test]
fn pdf_paginates_long_reports_without_error() {
    let foods: Vec<FoodEntry> = (0..120)
        .map(|index| food(&format!("Makanan ke-{index}")))
        .collect();
    let bytes = export::render_pdf(request(foods)).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Type /Page"));
    assert!(text.matches("/Type /Page ").count() > 1 || text.contains("/Count"));
    assert!(text.contains("Makanan ke-119"));
}

#[test]
fn macro_alias_uses_first_match_like_dashboard_not_sum() {
    let mut item = food("Nasi");
    item.nutrients = HashMap::from([
        ("Energi (kkal)".to_string(), 100.0),
        ("Energi".to_string(), 50.0),
        ("carbohydrate".to_string(), 20.0),
    ]);
    let bytes = export::render_rtf(request(vec![item]), b"prefix ===================================================================== suffix \\par }{\\*\\themedata").unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("50,0 kcal (100 %)"), "energy should be 50 like dashboard");
    assert!(!output.contains("150,0"), "energy must not be double counted");
}

#[test]
fn pdf_uses_times_new_roman_like_original_template() {
    let bytes = export::render_pdf(request(vec![food("Nasi")])).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/BaseFont /Times-Roman"));
    assert!(text.contains("/BaseFont /Times-Bold"));
}

#[test]
fn docx_uses_times_new_roman_like_original_template() {
    let bytes = export::render_docx(request(vec![food("Nasi")])).unwrap();
    let (entries, _) = export::test_support::parse_stored_zip(&bytes);
    let document = entries
        .iter()
        .find(|(name, _)| name == "word/document.xml")
        .map(|(_, data)| String::from_utf8_lossy(data).to_string())
        .unwrap();
    assert!(document.contains("Times New Roman"));
}

#[test]
fn zero_targets_do_not_invent_recommendation_numbers() {
    let mut req = request(vec![food("Nasi")]);
    req.targets = Some(NutritionTargets {
        kcal: 0.0,
        carbs: 0.0,
        protein: 0.0,
        fat: 0.0,
    });
    let bytes = export::render_rtf(req, include_bytes!("../../Assets/template.rtf")).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    // Target project 0 tidak boleh diganti angka AKG rekaan (1900/332/47/73).
    assert!(!output.contains("1900,0"));
    assert!(!output.contains("332,0"));
    assert!(!output.contains("47,0"));
    assert!(!output.contains("73,0"));
    assert!(output.contains("energy\\tab 100,0 kcal\\tab -\\tab - \\par "));
}

#[test]
fn report_numbers_match_dashboard_totals() {
    let mut rst = food("Nasi Putih");
    // Data nyata dari database: kunci kanonik, sajian 100 g, dimakan 150 g.
    rst.amount = 150.0;
    rst.serving_size = 100.0;
    rst.nutrients = HashMap::from([
        ("energi".to_string(), 129.0),
        ("karbohidrat total".to_string(), 28.2),
        ("protein".to_string(), 2.7),
        ("lemak total".to_string(), 0.3),
    ]);
    let mut egg = food("Telur Rebus");
    egg.amount = 50.0;
    egg.serving_size = 100.0;
    egg.nutrients = HashMap::from([
        ("energi".to_string(), 154.0),
        ("karbohidrat total".to_string(), 1.1),
        ("protein".to_string(), 12.6),
        ("lemak total".to_string(), 10.6),
    ]);
    let mut req = request(vec![rst, egg]);
    req.targets = Some(NutritionTargets {
        kcal: 2000.0,
        carbs: 300.0,
        protein: 50.0,
        fat: 70.0,
    });
    // Nilai acuan sama seperti calculateTotals() Dashboard.
    let expected_energy = 129.0 * 1.5 + 154.0 * 0.5;
    let expected_carbs = 28.2 * 1.5 + 1.1 * 0.5;
    let expected_protein = 2.7 * 1.5 + 12.6 * 0.5;
    let expected_fat = 0.3 * 1.5 + 10.6 * 0.5;
    let rtf = String::from_utf8(
        export::render_rtf(req.clone(), include_bytes!("../../Assets/template.rtf")).unwrap(),
    )
    .unwrap();
    assert!(rtf.contains(&format!(
        "energy {} kcal (100 %)",
        format!("{expected_energy:.1}").replace('.', ",")
    )));
    assert!(rtf.contains(&format!(
        "protein\\tab {expected_protein:.1} g({:.0}%)",
        expected_protein * 4.0 / expected_energy * 100.0
    ).replace('.', ",")));
    let pdf = String::from_utf8(export::render_pdf(req.clone()).unwrap()).unwrap();
    assert!(pdf.contains(&format!("{expected_energy:.1}", ).replace('.', ",")));
    let docx_bytes = export::render_docx(req.clone()).unwrap();
    let (entries, _) = export::test_support::parse_stored_zip(&docx_bytes);
    let document = entries
        .iter()
        .find(|(name, _)| name == "word/document.xml")
        .map(|(_, data)| String::from_utf8_lossy(data).to_string())
        .unwrap();
    assert!(document.contains(&format!("{expected_carbs:.1}").replace('.', ",")));
    assert!(document.contains(&format!("{expected_fat:.1}").replace('.', ",")));
}

#[test]
fn validate_selected_path_accepts_matching_format_extension() {
    assert!(export::validate_selected_path(Some(Path::new("r.pdf")), ExportFormat::Pdf).is_ok());
    assert!(export::validate_selected_path(Some(Path::new("r.docx")), ExportFormat::Docx).is_ok());
    assert!(export::validate_selected_path(Some(Path::new("r.rtf")), ExportFormat::Rtf).is_ok());
    let error = export::validate_selected_path(Some(Path::new("r.rtf")), ExportFormat::Pdf)
        .unwrap_err();
    assert!(matches!(error, AppError::Export(message) if message.contains(".pdf")));
}
