use web_sys::MouseEvent;
use yew::{Callback, function_component, html, Html, Properties, use_context, use_state, UseReducerDispatcher};
use yew_hooks::use_async;

use common::model::{TodoStatus, UpdateTodoRequest};

use crate::state::{TodoAction, TodoContext, TodoState};
use crate::todo_client;
use crate::icon;

#[derive(Properties, PartialEq, Clone)]
pub struct TodoControlProps {
    pub id: i32,
    pub status: TodoStatus,
    pub editing: bool,
    pub on_edit: Callback<MouseEvent>,
    pub on_save_editing: Callback<MouseEvent>,
    pub dispatcher: UseReducerDispatcher<TodoState>,
}

#[function_component(TodoControl)]
pub fn todo_control(
    TodoControlProps { id, status, editing, on_edit, dispatcher, on_save_editing }: &TodoControlProps
) -> Html {
    let status = status.clone();
    let on_save_editing = on_save_editing.clone();
    let editing = *editing;

    let update_todo_param = use_state(|| Option::<TodoStatus>::None);
    let update_todo_status = {
        let id = id.clone() as i32;
        let status_to_update = update_todo_param.clone().clone();
        let d = dispatcher.clone();
        use_async(async move {
            let result = todo_client().update_todo(id, UpdateTodoRequest {
                content: None,
                status: *status_to_update,
            }).await;

            d.dispatch(TodoAction::Refresh);

            result.map_err(|e| e.to_string())
        })
    };

    let context = use_context::<TodoContext>().expect("no ctx found");
    let update_status = {
        |status: TodoStatus| {
            let id = id.clone();
            let d = dispatcher.clone();
            let param = update_todo_param.clone();
            let update_todo_status = update_todo_status.clone();
            Callback::from(move |_| {
                if context.enable_remote {
                    param.set(Some(status));
                    update_todo_status.run();
                } else {
                    d.dispatch(TodoAction::Update(id, UpdateTodoRequest {
                        content: None,
                        status: Some(status),
                    }))
                }
            })
        }
    };

    let checked = status == TodoStatus::Done;
    let show_todo = status == TodoStatus::Todo || checked;

    struct IconButton {
        visible: bool,
        on_click: Callback<MouseEvent>,
        tooltip: String,
        color: String,
        icon: Html,
    }

    let icon_buttons = [
        IconButton {
            visible: editing,
            on_click: on_save_editing,
            tooltip: "Save".to_string(),
            color: "has-text-gray".to_string(),
            icon: html! { <icon::EditSave /> },
        },
        IconButton {
            visible: editing,
            on_click: on_edit.clone(),
            tooltip: "Cancel".to_string(),
            color: "has-text-gray".to_string(),
            icon: html! { <icon::EditCancel /> },
        },
        IconButton {
            visible: !editing && show_todo && !checked,
            on_click: update_status(TodoStatus::Done),
            tooltip: "Done".to_string(),
            color: "has-text-success".to_string(),
            icon: html! { <icon::Check /> },
        },
        IconButton {
            visible: !editing && show_todo && checked,
            on_click: update_status(TodoStatus::Todo),
            tooltip: "Undo".to_string(),
            color: "has-text-info".to_string(),
            icon: html! { <icon::Undo /> },
        },
        IconButton {
            visible: !editing && show_todo && !checked,
            on_click: on_edit.clone(),
            tooltip: "Edit".to_string(),
            color: "has-text-gray".to_string(),
            icon: html! { <icon::Edit /> },
        },
        IconButton {
            visible: !editing && show_todo,
            on_click: update_status(TodoStatus::Archived),
            tooltip: "Archive".to_string(),
            color: "has-text-gray".to_string(),
            icon: html! { <icon::Archive /> },
        },
        IconButton {
            visible: !editing && status == TodoStatus::Archived,
            on_click: update_status(TodoStatus::Deleted),
            tooltip: "Delete".to_string(),
            color: "has-text-dark".to_string(),
            icon: html! { <icon::Delete /> },
        },
    ].into_iter().map(|it| {
        html! {
            if it.visible {
                <a class={ format!("button is-text is-small {}", it.color)}
                    data-tooltip={it.tooltip} onclick={it.on_click}>
                    <span class="icon is-small">{ it.icon }</span>
                </a>
            }
        }
    }).collect::<Html>();

    html! {
        <>
            {icon_buttons}
        </>
    }
}
