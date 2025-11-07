use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use ts_rs::TS;
use uuid::Uuid;

/// Status of a task specification in the review workflow
#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS)]
#[sqlx(type_name = "task_spec_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum TaskSpecStatus {
    Draft,      // Initial state - spec is being written
    Review,     // Spec is ready for review
    Approved,   // Spec has been approved and ready for execution
    Rejected,   // Spec was rejected and needs revision
}

/// A user story for the specification
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UserStory {
    pub workflow: String,       // The user workflow or action
    pub problem_solved: String, // What problem this solves
}

/// Complete task specification following Agent OS format
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskSpec {
    pub id: Uuid,
    pub task_id: Uuid,

    // Spec content
    pub overview: String,                    // 1-2 sentence goal and objective
    pub user_stories: String,                // JSON array of UserStory
    pub scope: String,                       // JSON array of scope items
    pub out_of_scope: Option<String>,        // Items out of scope
    pub deliverables: String,                // Expected deliverable

    // Status and approval
    pub status: TaskSpecStatus,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new task spec
#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct CreateTaskSpec {
    pub task_id: Uuid,
    pub overview: String,
    pub user_stories: Vec<UserStory>,
    pub scope: Vec<String>,
    pub out_of_scope: Option<String>,
    pub deliverables: String,
}

/// Data for updating an existing task spec
#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct UpdateTaskSpec {
    pub overview: Option<String>,
    pub user_stories: Option<Vec<UserStory>>,
    pub scope: Option<Vec<String>>,
    pub out_of_scope: Option<String>,
    pub deliverables: Option<String>,
}

/// Data for updating spec status (approve/reject)
#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct UpdateTaskSpecStatus {
    pub status: TaskSpecStatus,
    pub approved_by: Option<String>,
    pub rejection_reason: Option<String>,
}

impl TaskSpec {
    /// Find a spec by task ID
    pub async fn find_by_task_id(
        pool: &SqlitePool,
        task_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            TaskSpec,
            r#"SELECT
                id as "id!: Uuid",
                task_id as "task_id!: Uuid",
                overview,
                user_stories,
                scope,
                out_of_scope,
                deliverables,
                status as "status!: TaskSpecStatus",
                approved_by,
                approved_at as "approved_at?: DateTime<Utc>",
                rejection_reason,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM task_specs
            WHERE task_id = $1"#,
            task_id
        )
        .fetch_optional(pool)
        .await
    }

    /// Find a spec by its ID
    pub async fn find_by_id(
        pool: &SqlitePool,
        id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            TaskSpec,
            r#"SELECT
                id as "id!: Uuid",
                task_id as "task_id!: Uuid",
                overview,
                user_stories,
                scope,
                out_of_scope,
                deliverables,
                status as "status!: TaskSpecStatus",
                approved_by,
                approved_at as "approved_at?: DateTime<Utc>",
                rejection_reason,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM task_specs
            WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    /// Create a new task spec
    pub async fn create(
        pool: &SqlitePool,
        data: &CreateTaskSpec,
        spec_id: Uuid,
    ) -> Result<Self, sqlx::Error> {
        // Serialize complex fields to JSON
        let user_stories_json = serde_json::to_string(&data.user_stories)
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;
        let scope_json = serde_json::to_string(&data.scope)
            .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        sqlx::query_as!(
            TaskSpec,
            r#"INSERT INTO task_specs (
                id, task_id, overview, user_stories, scope,
                out_of_scope, deliverables, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id as "id!: Uuid",
                task_id as "task_id!: Uuid",
                overview,
                user_stories,
                scope,
                out_of_scope,
                deliverables,
                status as "status!: TaskSpecStatus",
                approved_by,
                approved_at as "approved_at?: DateTime<Utc>",
                rejection_reason,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>""#,
            spec_id,
            data.task_id,
            data.overview,
            user_stories_json,
            scope_json,
            data.out_of_scope,
            data.deliverables,
            TaskSpecStatus::Draft as TaskSpecStatus
        )
        .fetch_one(pool)
        .await
    }

    /// Update an existing task spec
    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        data: &UpdateTaskSpec,
    ) -> Result<Self, sqlx::Error> {
        // Get the current spec to preserve unchanged fields
        let current = Self::find_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let user_stories_json = if let Some(ref stories) = data.user_stories {
            serde_json::to_string(stories)
                .map_err(|e| sqlx::Error::Encode(Box::new(e)))?
        } else {
            current.user_stories.clone()
        };

        let scope_json = if let Some(ref scope_items) = data.scope {
            serde_json::to_string(scope_items)
                .map_err(|e| sqlx::Error::Encode(Box::new(e)))?
        } else {
            current.scope.clone()
        };

        let overview = data.overview.clone().unwrap_or(current.overview);
        let out_of_scope = data.out_of_scope.clone().or(current.out_of_scope);
        let deliverables = data.deliverables.clone().unwrap_or(current.deliverables);

        sqlx::query_as!(
            TaskSpec,
            r#"UPDATE task_specs
            SET overview = $2,
                user_stories = $3,
                scope = $4,
                out_of_scope = $5,
                deliverables = $6,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING
                id as "id!: Uuid",
                task_id as "task_id!: Uuid",
                overview,
                user_stories,
                scope,
                out_of_scope,
                deliverables,
                status as "status!: TaskSpecStatus",
                approved_by,
                approved_at as "approved_at?: DateTime<Utc>",
                rejection_reason,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            overview,
            user_stories_json,
            scope_json,
            out_of_scope,
            deliverables
        )
        .fetch_one(pool)
        .await
    }

    /// Update the status of a spec (approve/reject/etc)
    pub async fn update_status(
        pool: &SqlitePool,
        id: Uuid,
        data: &UpdateTaskSpecStatus,
    ) -> Result<Self, sqlx::Error> {
        let approved_at = if data.status == TaskSpecStatus::Approved {
            Some(Utc::now())
        } else {
            None
        };

        let status_value = data.status.clone() as TaskSpecStatus;

        sqlx::query_as!(
            TaskSpec,
            r#"UPDATE task_specs
            SET status = $2,
                approved_by = $3,
                approved_at = $4,
                rejection_reason = $5,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING
                id as "id!: Uuid",
                task_id as "task_id!: Uuid",
                overview,
                user_stories,
                scope,
                out_of_scope,
                deliverables,
                status as "status!: TaskSpecStatus",
                approved_by,
                approved_at as "approved_at?: DateTime<Utc>",
                rejection_reason,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            status_value,
            data.approved_by,
            approved_at,
            data.rejection_reason
        )
        .fetch_one(pool)
        .await
    }

    /// Delete a task spec
    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM task_specs WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Check if a task has a spec
    pub async fn exists_for_task(
        pool: &SqlitePool,
        task_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT id as \"id!: Uuid\" FROM task_specs WHERE task_id = $1",
            task_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(result.is_some())
    }

    /// Parse user stories from JSON string
    pub fn parse_user_stories(&self) -> Result<Vec<UserStory>, serde_json::Error> {
        serde_json::from_str(&self.user_stories)
    }

    /// Parse scope items from JSON string
    pub fn parse_scope(&self) -> Result<Vec<String>, serde_json::Error> {
        serde_json::from_str(&self.scope)
    }
}
