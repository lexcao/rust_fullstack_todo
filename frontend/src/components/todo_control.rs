use web_sys::MouseEvent;
use yew::{Callback, function_component, html, Html, Properties};

use common::model::TodoStatus;

use crate::icons;

#[derive(Properties, PartialEq, Clone)]
pub struct TodoControlProps {
    pub status: TodoStatus,
    pub editing: bool,
    pub content: String,
    pub on_edit: Callback<MouseEvent>,
    pub on_save_editing: Callback<MouseEvent>,
    pub on_status: Callback<TodoStatus>,
}

struct IconButton {
    visible: bool,
    on_click: Callback<MouseEvent>,
    label: &'static str,
    tone: &'static str,
    icon: Html,
}

/// 每条任务右侧的操作按钮。只负责展示，状态更新由 TodoDetails 处理。
#[function_component(TodoControl)]
pub fn todo_control(
    TodoControlProps { status, editing, content, on_edit, on_save_editing, on_status }: &TodoControlProps
) -> Html {
    let status = *status;
    let editing = *editing;
    let set_status = |next: TodoStatus| {
        let on_status = on_status.clone();
        Callback::from(move |_: MouseEvent| on_status.emit(next))
    };
    let active = status == TodoStatus::Todo || status == TodoStatus::Done;

    let buttons = [
        IconButton {
            visible: editing,
            on_click: on_save_editing.clone(),
            label: "Save",
            tone: "is-primary",
            icon: html! { <icons::EditSave /> },
        },
        IconButton {
            visible: editing,
            on_click: on_edit.clone(),
            label: "Cancel",
            tone: "",
            icon: html! { <icons::EditCancel /> },
        },
        IconButton {
            visible: !editing && status == TodoStatus::Todo,
            on_click: on_edit.clone(),
            label: "Edit",
            tone: "",
            icon: html! { <icons::Edit /> },
        },
        IconButton {
            visible: !editing && active,
            on_click: set_status(TodoStatus::Archived),
            label: "Archive",
            tone: "",
            icon: html! { <icons::Archive /> },
        },
        IconButton {
            visible: !editing && status == TodoStatus::Archived,
            on_click: set_status(TodoStatus::Deleted),
            label: "Move to bin",
            tone: "is-danger",
            icon: html! { <icons::Delete /> },
        },
    ];

    buttons.into_iter().filter(|it| it.visible).map(|it| {
        html! {
            <button type="button" class={format!("icon-btn {}", it.tone)}
                data-tip={it.label}
                aria-label={format!("{}: {}", it.label, content)}
                onclick={it.on_click}>
                { it.icon }
            </button>
        }
    }).collect::<Html>()
}
