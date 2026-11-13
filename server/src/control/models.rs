//! Implements model configuration endpoints.
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    model::{ModelConfig, ModelConfigInput},
    Result,
};

use super::{
    ControlService, DiscoveredModels, LegacyModelImportPreview, LegacyModelImportResult,
    ModelConnectivityResult, ModelDiscoveryInput, ModelImportPreview, ModelImportResult,
};

#[derive(Deserialize)]
pub struct SaveModels {
    pub models: Vec<ModelConfigInput>,
}

#[derive(Deserialize)]
pub struct ModelOrder {
    pub model_hashes: Vec<String>,
}

#[derive(Deserialize)]
pub struct ImportModels {
    pub models: Vec<ModelConfigInput>,
}

pub async fn list(State(service): State<ControlService>) -> Result<Json<Vec<ModelConfig>>> {
    Ok(Json(service.models().await?))
}

/// 以 JSON 附件导出全部模型配置,包含 API Key、扩展参数和自定义请求头,可直接用于导入还原。
pub async fn export(State(service): State<ControlService>) -> Result<axum::response::Response> {
    let models = service.models().await?;
    let exported = models.iter().map(ModelConfig::input).collect::<Vec<_>>();
    let body = serde_json::to_vec_pretty(&serde_json::json!({
        "exported_at_ms": crate::store::now_ms(),
        "model_count": exported.len(),
        "models": exported,
    }))?;
    Ok(axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            "attachment; filename=\"cursor-byok-models.json\"",
        )
        .body(axum::body::Body::from(body))
        .expect("static model export response"))
}

pub async fn create(
    State(service): State<ControlService>,
    Json(input): Json<SaveModels>,
) -> Result<(StatusCode, Json<Vec<ModelConfig>>)> {
    Ok((
        StatusCode::CREATED,
        Json(service.create_models(&input.models).await?),
    ))
}

pub async fn reorder(
    State(service): State<ControlService>,
    Json(input): Json<ModelOrder>,
) -> Result<Json<Vec<ModelConfig>>> {
    Ok(Json(service.reorder_models(&input.model_hashes).await?))
}

pub async fn remove(
    State(service): State<ControlService>,
    Path(model_hash): Path<String>,
) -> Result<StatusCode> {
    service.delete_model(&model_hash).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update(
    State(service): State<ControlService>,
    Path(model_hash): Path<String>,
    Json(input): Json<ModelConfigInput>,
) -> Result<Json<ModelConfig>> {
    Ok(Json(service.update_model(&model_hash, &input).await?))
}

pub async fn test(
    State(service): State<ControlService>,
    Path((model_hash, test_id)): Path<(String, String)>,
) -> Result<Json<ModelConnectivityResult>> {
    Ok(Json(service.test_model(&model_hash, &test_id).await?))
}

pub async fn cancel(
    State(service): State<ControlService>,
    Path((_model_hash, test_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    service.cancel_model_test(&test_id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn discover(
    State(service): State<ControlService>,
    Json(input): Json<ModelDiscoveryInput>,
) -> Result<Json<DiscoveredModels>> {
    Ok(Json(service.discover_models(&input).await?))
}

pub async fn preview_import(
    State(service): State<ControlService>,
    Json(input): Json<ImportModels>,
) -> Result<Json<ModelImportPreview>> {
    Ok(Json(service.preview_model_import(input.models).await?))
}

pub async fn import(
    State(service): State<ControlService>,
    Json(input): Json<ImportModels>,
) -> Result<Json<ModelImportResult>> {
    Ok(Json(service.import_models(input.models).await?))
}

pub async fn import_v0049(
    State(service): State<ControlService>,
) -> Result<Json<LegacyModelImportResult>> {
    Ok(Json(service.import_v0049_models().await?))
}

pub async fn preview_v0049(
    State(service): State<ControlService>,
) -> Result<Json<LegacyModelImportPreview>> {
    Ok(Json(service.preview_v0049_models().await?))
}
