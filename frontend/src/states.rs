use std::rc::Rc;
use gloo::storage::{LocalStorage, Storage};
use yew::Reducible;
use common::model::{TodoStatus, UpdateTodoRequest};
use crate::domain::{create_todo, Todo};

#[derive(Clone)]
pub enum TodoAction {
    Add(String),
    Update(i32, UpdateTodoRequest),
    ClearDeleted,
    Refresh,
}

#[derive(PartialEq, Clone)]
pub struct TodoState {
    pub locals: Vec<Todo>,
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
                let todo = create_todo(&locals.len(), &content);
                locals.insert(0, todo);

                locals
            }
            TodoAction::Update(id, update) => {
                let mut locals = self.locals.clone();
                if let Some(index) = locals.iter().position(|it| it.id == id) {
                    if let Some(content) = update.content {
                        locals[index].content = content
                    }
                    if let Some(status) = update.status {
                        locals[index].status = status
                    }
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
