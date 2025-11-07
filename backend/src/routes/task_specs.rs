use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    models::{
        api_response::ApiResponse,
        task::{Task, TaskStatus},
        task_spec::{CreateTaskSpec, TaskSpec, UpdateTaskSpec, UpdateTaskSpecStatus},
    },
};

/// Helper function to verify that a task belongs to the specified project
async fn verify_task_in_project(
    pool: &SqlitePool,
    project_id: Uuid,
    task_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    // Query the task and check if it belongs to the project
    let task = sqlx::query_as!(
        Task,
        r#"SELECT 
            id as "id!: Uuid",
            project_id as "project_id!: Uuid",
            title,
            description,
            status as "status!: TaskStatus",
            created_at as "created_at!: DateTime<Utc>",
            updated_at as "updated_at!: DateTime<Utc>"
        FROM tasks 
        WHERE id = $1"#,
        task_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to verify task: {}", e))),
        )
    })?;

    match task {
        Some(t) if t.project_id == project_id => Ok(()),
        Some(_) => Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Task does not belong to this project")),
        )),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Task not found")),
        )),
    }
}


/// Get spec for a specific task
pub async fn get_task_spec(
    State(state): State<AppState>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // Verify task belongs to project
    verify_task_in_project(&state.db_pool, project_id, task_id).await?;

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
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(mut payload): Json<CreateTaskSpec>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // Verify task belongs to project
    verify_task_in_project(&state.db_pool, project_id, task_id).await?;

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
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTaskSpec>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // Verify task belongs to project
    verify_task_in_project(&state.db_pool, project_id, task_id).await?;

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
/// 
/// # Security Note
/// Currently, this endpoint accepts `approved_by` from the client request body.
/// This is a security issue as any client can claim approval from an arbitrary person.
/// 
/// TODO: Once authentication is implemented:
/// 1. Derive `approved_by` from the authenticated user/session on the server
/// 2. Ignore or reject the client-provided `approved_by` value
/// 3. Consider adding validation to ensure only authorized users can approve specs
pub async fn update_task_spec_status(
    State(state): State<AppState>,
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTaskSpecStatus>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // Verify task belongs to project
    verify_task_in_project(&state.db_pool, project_id, task_id).await?;

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
    Path((project_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<()>>)> {
    // Verify task belongs to project
    verify_task_in_project(&state.db_pool, project_id, task_id).await?;

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
