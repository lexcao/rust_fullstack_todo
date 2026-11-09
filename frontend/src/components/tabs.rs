use yew::{Callback, classes, function_component, Html, html, Properties};

use common::model::{TodoResponse, TodoStatus};

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct StatusCounts {
    pub todo: usize,
    pub done: usize,
    pub archived: usize,
    pub deleted: usize,
}

impl StatusCounts {
    pub fn from_todos(todos: &[TodoResponse]) -> Self {
        todos.iter().fold(Self::default(), |mut acc, todo| {
            match todo.status {
                TodoStatus::Todo => acc.todo += 1,
                TodoStatus::Done => acc.done += 1,
                TodoStatus::Archived => acc.archived += 1,
                TodoStatus::Deleted => acc.deleted += 1,
            }
            acc
        })
    }

    pub fn get(&self, status: Option<TodoStatus>) -> usize {
        match status {
            None => self.todo + self.done + self.archived + self.deleted,
            Some(TodoStatus::Todo) => self.todo,
            Some(TodoStatus::Done) => self.done,
            Some(TodoStatus::Archived) => self.archived,
            Some(TodoStatus::Deleted) => self.deleted,
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct TabsProps {
    pub on_select: Callback<Option<TodoStatus>>,
    pub selected: Option<TodoStatus>,
    #[prop_or_default]
    pub counts: Option<StatusCounts>,
}

#[function_component(Tabs)]
pub fn tabs(TabsProps { on_select, selected, counts }: &TabsProps) -> Html {
    let tabs = [
        None,
        Some(TodoStatus::Todo),
        Some(TodoStatus::Done),
        Some(TodoStatus::Archived),
        Some(TodoStatus::Deleted),
    ];

    let items = tabs.iter().map(|status| {
        let status = *status;
        let active = status == *selected;
        let label = match status {
            None => "All",
            Some(TodoStatus::Todo) => "Open",
            Some(TodoStatus::Done) => "Done",
            Some(TodoStatus::Archived) => "Archived",
            Some(TodoStatus::Deleted) => "Bin",
        };
        let onclick = {
            let on_select = on_select.clone();
            Callback::from(move |_| on_select.emit(status))
        };
        html! {
            <button type="button" role="tab"
                class={classes!("tab", active.then_some("is-active"))}
                aria-selected={active.to_string()}
                {onclick}>
                <span>{ label }</span>
                if let Some(counts) = counts {
                    <span class="tab-count">{ counts.get(status) }</span>
                }
            </button>
        }
    }).collect::<Html>();

    html! {
        <nav class="tabs" role="tablist" aria-label="Filter tasks">
            { items }
        </nav>
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn todo(status: TodoStatus) -> TodoResponse {
        TodoResponse {
            namespace: "local".into(),
            id: 0,
            content: "t".into(),
            status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn counts_each_status_and_total() {
        let todos = vec![
            todo(TodoStatus::Todo),
            todo(TodoStatus::Todo),
            todo(TodoStatus::Done),
            todo(TodoStatus::Deleted),
        ];
        let counts = StatusCounts::from_todos(&todos);
        assert_eq!(counts.get(Some(TodoStatus::Todo)), 2);
        assert_eq!(counts.get(Some(TodoStatus::Done)), 1);
        assert_eq!(counts.get(Some(TodoStatus::Archived)), 0);
        assert_eq!(counts.get(None), 4);
    }
}
