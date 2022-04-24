use gloo::storage::{LocalStorage, Storage};

const KEY: &str = "rust_fullstack_todo.namespace";

pub fn get() -> String {
    LocalStorage::get(KEY)
        .unwrap_or_else(|_| {
            let value = generate();
            LocalStorage::set(KEY, value.clone()).unwrap();
            value
        })
}

fn generate() -> String {
    uuid::Uuid::new_v4().to_string()
}