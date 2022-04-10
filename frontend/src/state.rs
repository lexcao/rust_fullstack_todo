use std::collections::BTreeMap;
use std::rc::Rc;
use gloo::console::console_dbg;
use gloo::storage::{LocalStorage, Storage};
use yew::Reducible;
use crate::domain::{Todo, TodoStatus};

#[derive(Clone)]
pub enum TodoAction {
    Add(String),
    Edit(u128, String),
    UpdateStatus(u128, TodoStatus),
    ClearDeleted,
}

#[derive(PartialEq)]
pub struct TodoState {
    pub todos: BTreeMap<u128, Todo>,
}

impl TodoState {
    pub fn save_to_local(&self) {
        LocalStorage::set(KEY, self.todos.clone())
            .expect("failed to save")
    }
}

const KEY: &str = "rust_fullstack_todo.todos";

impl Default for TodoState {
    fn default() -> Self {
        Self { todos: LocalStorage::get(KEY).unwrap_or_else(|_| BTreeMap::new()) }
    }
}

impl Reducible for TodoState {
    type Action = TodoAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let next = match action {
            TodoAction::Add(content) => {
                let mut todos = self.todos.clone();

                let current_timestamp = instant::SystemTime::now()
                    .duration_since(instant::SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let id = current_timestamp;

                let mut new_todo = Todo::new(&content);
                new_todo.id = id;
                todos.insert(id, new_todo);
                todos
            }
            TodoAction::Edit(id, content) => {
                let mut todos = self.todos.clone();
                if let Some(t) = todos.get_mut(&id) {
                    t.content = content
                }
                todos
            }
            TodoAction::UpdateStatus(id, status) => {
                let mut todos = self.todos.clone();
                if let Some(t) = todos.get_mut(&id) {
                    t.status = status
                }
                todos
            }
            TodoAction::ClearDeleted => {
                self.todos.clone()
                    .into_iter()
                    .filter(|(_, v)| v.status != TodoStatus::Deleted)
                    .collect()
            }
        };

        Rc::new(Self { todos: next })
    }
}
