use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFile {
    pub version: u32,
    pub foods: Vec<ProjectFood>,
    pub meals: Vec<ProjectMeal>,
    pub targets: ProjectTargets,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFood {
    pub id: String,
    pub name: String,
    pub serving_size: f64,
    pub serving_unit: String,
    pub servings_per_container: f64,
    pub amount: f64,
    pub meal_time: String,
    pub nutrients: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectMeal {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectTargets {
    pub kcal: f64,
    pub carbs: f64,
    pub protein: f64,
    pub fat: f64,
}

pub async fn save(path: &Path, project: &ProjectFile) -> Result<(), AppError> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("nutri") {
        return Err(AppError::Validation(
            "project file must use .nutri extension".into(),
        ));
    }
    let bytes = serde_json::to_vec(project).map_err(|error| AppError::Io(error.to_string()))?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(path, bytes).await?;
    Ok(())
}

pub async fn load(path: &Path) -> Result<ProjectFile, AppError> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("nutri") {
        return Err(AppError::Validation(
            "project file must use .nutri extension".into(),
        ));
    }
    let bytes = tokio::fs::read(path).await?;
    let project: ProjectFile = serde_json::from_slice(&bytes)
        .map_err(|_| AppError::Validation("File proyek tidak valid".into()))?;
    if project.version != 1 {
        return Err(AppError::Validation("File proyek tidak didukung".into()));
    }
    let meal_ids = project
        .meals
        .iter()
        .map(|meal| meal.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    if project
        .foods
        .iter()
        .any(|food| !meal_ids.contains(food.meal_time.as_str()))
    {
        return Err(AppError::Validation("File proyek tidak valid".into()));
    }
    Ok(project)
}
