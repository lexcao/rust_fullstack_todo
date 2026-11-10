//! 待办规则。PATCH 的状态流转见 [`transition_allowed`]：
//! 只拒绝前端今天发不出来、而且现有契约/集成测试也没在用的边。

use anyhow::Result;
use common::model::TodoStatus;

use crate::repo::{Todo, TodoID, TodoRepo};

pub struct UpdateTodo {
    pub content: Option<String>,
    pub status: Option<TodoStatus>,
}

pub async fn get_todo(repo: &dyn TodoRepo, id: TodoID) -> Result<Todo> {
    repo.query_by_id(id).await
}

pub async fn list_todos(
    repo: &dyn TodoRepo,
    namespace: String,
    status: Option<TodoStatus>,
) -> Result<Vec<Todo>> {
    repo.query_todos(namespace, status).await
}

pub async fn create_todo(repo: &dyn TodoRepo, namespace: &str, content: &str) -> Result<Todo> {
    repo.insert_todo(Todo::create(namespace, content)).await
}

pub async fn update_todo(repo: &dyn TodoRepo, id: TodoID, to_update: UpdateTodo) -> Result<Todo> {
    let mut found = repo.query_by_id(id).await?;

    if let Some(content) = to_update.content {
        found.content = content;
    }
    if let Some(to_status) = to_update.status {
        if !transition_allowed(found.status, to_status) {
            return Err(TodoError::InvalidStatusTransition {
                from: status_label(found.status),
                to: status_label(to_status),
            }
            .into());
        }
        found.status = to_status;
    }

    repo.update_todo(found).await
}

/// 前端云模式实际会发的边（`frontend/src/components/todo_detail.rs` 和 `todo_control.rs`）：
/// - 勾选：`todo ↔ done`（只有这两种状态看得到复选框）
/// - 归档按钮：`todo` 和 `done` → `archived`（`active` 含这两种）
/// - 移入回收站：`archived` → `deleted`
///
/// 另外两条不是按钮，但现有契约测试和 `todo_integration` 会把**新建的** `todo`
/// 直接 PATCH 成每一个状态（包括 `todo → deleted` 和写成自己）。拦掉它们会让
/// 那些没标「P3.4 改」的断言失败，所以保留。任意状态写成自己也放行（无操作）。
///
/// 不放行：`done → deleted`（界面必须先归档）、从 `archived` 回到 `todo`/`done`、
/// 以及任何离开 `deleted` 的状态变化。只改内容不改状态，不经过这里。
pub fn transition_allowed(from: TodoStatus, to: TodoStatus) -> bool {
    from == to
        || matches!(
            (from, to),
            (TodoStatus::Todo, TodoStatus::Done)
                | (TodoStatus::Todo, TodoStatus::Archived)
                | (TodoStatus::Todo, TodoStatus::Deleted)
                | (TodoStatus::Done, TodoStatus::Todo)
                | (TodoStatus::Done, TodoStatus::Archived)
                | (TodoStatus::Archived, TodoStatus::Deleted)
        )
}

pub fn status_label(status: TodoStatus) -> &'static str {
    match status {
        TodoStatus::Todo => "todo",
        TodoStatus::Done => "done",
        TodoStatus::Archived => "archived",
        TodoStatus::Deleted => "deleted",
    }
}

pub async fn clear_todos(repo: &dyn TodoRepo, namespace: String, ids: Vec<i32>) -> Result<()> {
    repo.clear_todos(namespace, ids).await?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum TodoError {
    #[error("invalid status transition: {from} -> {to}")]
    InvalidStatusTransition {
        from: &'static str,
        to: &'static str,
    },
    #[error("invalid status from str {0}")]
    InvalidStatusFromStr(String),
}

pub async fn toggle_todo(repo: &dyn TodoRepo, todo: Todo) -> Result<Todo> {
    let mut todo = repo.query_by_id(todo.id).await?;

    let new_status = match todo.status {
        TodoStatus::Todo => TodoStatus::Done,
        TodoStatus::Done => TodoStatus::Todo,
        current => {
            return Err(TodoError::InvalidStatusTransition {
                from: status_label(current),
                to: status_label(TodoStatus::Todo),
            }
            .into());
        }
    };

    todo.status = new_status;
    repo.update_todo(todo).await
}

pub async fn archive_todo(repo: &dyn TodoRepo, todo: Todo) -> Result<Todo> {
    let mut found = repo.query_by_id(todo.id).await?;

    let new_status = match found.status {
        TodoStatus::Done => TodoStatus::Archived,
        current => {
            return Err(TodoError::InvalidStatusTransition {
                from: status_label(current),
                to: status_label(TodoStatus::Archived),
            }
            .into());
        }
    };

    found.status = new_status;
    repo.update_todo(found).await
}

pub async fn delete_todo(repo: &dyn TodoRepo, todo: Todo) -> Result<Todo> {
    let mut found = repo.query_by_id(todo.id).await?;

    let new_status = match found.status {
        TodoStatus::Archived => TodoStatus::Deleted,
        current => {
            return Err(TodoError::InvalidStatusTransition {
                from: status_label(current),
                to: status_label(TodoStatus::Deleted),
            }
            .into());
        }
    };

    found.status = new_status;
    repo.update_todo(found).await
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [TodoStatus; 4] = [
        TodoStatus::Todo,
        TodoStatus::Done,
        TodoStatus::Archived,
        TodoStatus::Deleted,
    ];

    #[test]
    fn transition_table_matches_the_frontend_and_the_locked_contract() {
        let allowed = [
            (TodoStatus::Todo, TodoStatus::Todo),
            (TodoStatus::Todo, TodoStatus::Done),
            (TodoStatus::Todo, TodoStatus::Archived),
            (TodoStatus::Todo, TodoStatus::Deleted),
            (TodoStatus::Done, TodoStatus::Todo),
            (TodoStatus::Done, TodoStatus::Done),
            (TodoStatus::Done, TodoStatus::Archived),
            (TodoStatus::Archived, TodoStatus::Archived),
            (TodoStatus::Archived, TodoStatus::Deleted),
            (TodoStatus::Deleted, TodoStatus::Deleted),
        ];
        for from in ALL {
            for to in ALL {
                let ok = allowed.contains(&(from, to));
                assert_eq!(
                    transition_allowed(from, to),
                    ok,
                    "{} -> {}",
                    status_label(from),
                    status_label(to)
                );
            }
        }
    }
}
