//! Implements local application control endpoints.
use axum::{extract::State, Json};

use crate::{
    local_app::{
        CursorHarnessStatus, CursorInstallPathInput, CursorLanguageInput, CursorLanguageStatus,
        SetEnabled,
    },
    Result,
};

use super::ControlService;

pub async fn status(State(service): State<ControlService>) -> Result<Json<CursorHarnessStatus>> {
    Ok(Json(service.cursor_harness().status().await?))
}

pub async fn initialize_ca(
    State(service): State<ControlService>,
) -> Result<Json<CursorHarnessStatus>> {
    Ok(Json(service.cursor_harness().initialize_ca().await?))
}

pub async fn set_enabled(
    State(service): State<ControlService>,
    Json(input): Json<SetEnabled>,
) -> Result<Json<CursorHarnessStatus>> {
    Ok(Json(
        service.cursor_harness().set_enabled(input.enabled).await?,
    ))
}

pub async fn language(
    State(service): State<ControlService>,
) -> Result<Json<CursorLanguageStatus>> {
    Ok(Json(service.cursor_harness().language_status().await?))
}

pub async fn set_language(
    State(service): State<ControlService>,
    Json(input): Json<CursorLanguageInput>,
) -> Result<Json<CursorLanguageStatus>> {
    Ok(Json(service.cursor_harness().set_language(input).await?))
}

pub async fn set_language_install_path(
    State(service): State<ControlService>,
    Json(input): Json<CursorInstallPathInput>,
) -> Result<Json<CursorLanguageStatus>> {
    Ok(Json(
        service
            .cursor_harness()
            .set_language_install_path(input)
            .await?,
    ))
}
