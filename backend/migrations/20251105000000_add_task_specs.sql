-- Add task_specs table for Agent OS spec mode
-- Following the Agent OS specification format

CREATE TABLE task_specs (
    id              BLOB PRIMARY KEY NOT NULL,
    task_id         BLOB NOT NULL UNIQUE,

    -- Spec content following Agent OS format
    overview        TEXT NOT NULL,              -- 1-2 sentence goal and objective
    user_stories    TEXT NOT NULL,              -- JSON array of 1-3 user stories
    scope           TEXT NOT NULL,              -- JSON array of 1-5 features in scope
    out_of_scope    TEXT,                       -- Items explicitly out of scope
    deliverables    TEXT NOT NULL,              -- Expected deliverable description

    -- Spec status workflow
    status          TEXT NOT NULL DEFAULT 'draft'
                       CHECK (status IN ('draft', 'review', 'approved', 'rejected')),

    -- Approval tracking
    approved_by     TEXT,                       -- Who approved the spec
    approved_at     TEXT,                       -- When it was approved
    rejection_reason TEXT,                      -- Why it was rejected if applicable

    created_at      TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),

    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Index for faster lookups
CREATE INDEX idx_task_specs_task_id ON task_specs(task_id);
CREATE INDEX idx_task_specs_status ON task_specs(status);
