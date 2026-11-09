use std::rc::Rc;

use chrono::Utc;
use gloo::storage::{LocalStorage, Storage};
use yew::Reducible;

use common::model::{TodoResponse, TodoStatus, UpdateTodoRequest};

#[derive(Clone)]
pub enum TodoAction {
    Add(String),
    Update(i32, UpdateTodoRequest),
    ClearDeleted,
    Refresh,
}

#[derive(PartialEq, Clone)]
pub struct TodoState {
    pub locals: Vec<TodoResponse>,
    pub refresh: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TodoContext {
    pub enable_remote: bool,
}

impl TodoState {
    pub fn save_to_local(&self) {
        LocalStorage::set(KEY, self.locals.clone())
            .expect("failed to save");
    }
}

const KEY: &str = "rust_fullstack_todo.todos";

impl Default for TodoState {
    fn default() -> Self {
        Self {
            locals: LocalStorage::get(KEY).unwrap_or_else(|_| Vec::new()),
            refresh: true,
        }
    }
}

impl Reducible for TodoState {
    type Action = TodoAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let next = match action {
            TodoAction::Add(content) => {
                let mut locals = self.locals.clone();
                let todo = create_todo(next_id(&locals), &content);
                locals.insert(0, todo);

                locals
            }
            TodoAction::Update(id, update) => {
                let mut locals = self.locals.clone();
                if let Some(index) = locals.iter().position(|it| it.id == id) {
                    if let Some(content) = update.content {
                        locals[index].content = content;
                    }
                    if let Some(status) = update.status {
                        locals[index].status = status;
                    }
                    locals[index].updated_at = Utc::now();
                }

                locals
            }
            TodoAction::ClearDeleted => {
                self.locals.clone()
                    .into_iter()
                    .filter(|it| it.status != TodoStatus::Deleted)
                    .collect()
            }
            _ => { self.locals.clone() }
        };

        Rc::new(Self {
            locals: next,
            refresh: !self.refresh,
        })
    }
}

/// 本地任务的新 id：取现有最大 id + 1。
/// 之前用列表长度，清空回收站后会和已有任务撞 id。
fn next_id(locals: &[TodoResponse]) -> i32 {
    locals.iter().map(|it| it.id).max().map_or(0, |max| max + 1)
}

fn create_todo(id: i32, content: &str) -> TodoResponse {
    TodoResponse {
        namespace: "local".to_string(),
        id,
        status: TodoStatus::Todo,
        content: content.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn todo(id: i32, status: TodoStatus) -> TodoResponse {
        TodoResponse { status, ..create_todo(id, "t") }
    }

    #[test]
    fn next_id_starts_at_zero() {
        assert_eq!(next_id(&[]), 0);
    }

    #[test]
    fn next_id_does_not_collide_after_clearing() {
        // 0,1,2 -> 删除 1 -> 剩 0,2；用长度会得到 2（撞 id），应得到 3
        let locals = vec![todo(2, TodoStatus::Todo), todo(0, TodoStatus::Done)];
        assert_eq!(next_id(&locals), 3);
    }
}
