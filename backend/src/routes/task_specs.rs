use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    models::{
        api_response::ApiResponse,
        task_spec::{CreateTaskSpec, TaskSpec, UpdateTaskSpec, UpdateTaskSpecStatus},
    },
};

/// Get spec for a specific task
pub async fn get_task_spec(
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    match TaskSpec::find_by_task_id(&state.db_pool, task_id).await {
        Ok(Some(spec)) => Ok(Json(ApiResponse::success(spec))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Spec not found for this task")),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to fetch spec: {}", e))),
        )),
    }
}

/// Create a new task spec
pub async fn create_task_spec(
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(mut payload): Json<CreateTaskSpec>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // Ensure the task_id in the path matches the payload
    payload.task_id = task_id;

    // Check if spec already exists for this task
    match TaskSpec::exists_for_task(&state.db_pool, task_id).await {
        Ok(true) => {
            return Err((
                StatusCode::CONFLICT,
                Json(ApiResponse::error("Spec already exists for this task")),
            ))
        }
        Ok(false) => {}
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to check spec existence: {}",
                    e
                ))),
            ))
        }
    }

    match TaskSpec::create(&state.db_pool, &payload, Uuid::new_v4()).await {
        Ok(spec) => Ok((StatusCode::CREATED, Json(ApiResponse::success(spec)))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to create spec: {}", e))),
        )),
    }
}

/// Update an existing task spec
pub async fn update_task_spec(
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTaskSpec>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // First, find the spec for this task
    let spec = match TaskSpec::find_by_task_id(&state.db_pool, task_id).await {
        Ok(Some(spec)) => spec,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Spec not found for this task")),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to fetch spec: {}", e))),
            ))
        }
    };

    match TaskSpec::update(&state.db_pool, spec.id, &payload).await {
        Ok(updated_spec) => Ok(Json(ApiResponse::success(updated_spec))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to update spec: {}", e))),
        )),
    }
}

/// Update the status of a task spec (approve/reject/review)
pub async fn update_task_spec_status(
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTaskSpecStatus>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // First, find the spec for this task
    let spec = match TaskSpec::find_by_task_id(&state.db_pool, task_id).await {
        Ok(Some(spec)) => spec,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Spec not found for this task")),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to fetch spec: {}", e))),
            ))
        }
    };

    match TaskSpec::update_status(&state.db_pool, spec.id, &payload).await {
        Ok(updated_spec) => Ok(Json(ApiResponse::success(updated_spec))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to update spec status: {}",
                e
            ))),
        )),
    }
}

/// Delete a task spec
pub async fn delete_task_spec(
    State(state): State<AppState>,
    Path((_project_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // First, find the spec for this task
    let spec = match TaskSpec::find_by_task_id(&state.db_pool, task_id).await {
        Ok(Some(spec)) => spec,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Spec not found for this task")),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to fetch spec: {}", e))),
            ))
        }
    };

    match TaskSpec::delete(&state.db_pool, spec.id).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, Json(ApiResponse::success(())))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to delete spec: {}", e))),
        )),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/:project_id/tasks/:task_id/spec",
            get(get_task_spec)
                .post(create_task_spec)
                .put(update_task_spec)
                .delete(delete_task_spec),
        )
        .route(
            "/projects/:project_id/tasks/:task_id/spec/status",
            post(update_task_spec_status),
        )
}
