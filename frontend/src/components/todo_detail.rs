use std::rc::Rc;

use web_sys::{HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::{Callback, classes, function_component, Html, html, Properties, use_context, use_effect_with, use_node_ref, use_state, UseReducerDispatcher};
use yew_hooks::use_async;

use common::model::{TodoResponse, TodoStatus, UpdateTodoRequest};
use crate::app::todo_client;

use crate::components::todo_control::*;
use crate::states::{TodoAction, TodoContext, TodoState};

#[derive(Properties, PartialEq, Clone)]
pub struct TodoDetailsProps {
    pub todo: TodoResponse,
    pub index: usize,
    pub dispatcher: UseReducerDispatcher<TodoState>,
}

fn status_class(status: TodoStatus) -> &'static str {
    match status {
        TodoStatus::Todo => "is-todo",
        TodoStatus::Done => "is-done",
        TodoStatus::Archived => "is-archived",
        TodoStatus::Deleted => "is-deleted",
    }
}

#[function_component(TodoDetails)]
pub fn todo_details(TodoDetailsProps { todo, index, dispatcher }: &TodoDetailsProps) -> Html {
    let editing = use_state(|| false);
    let context = use_context::<TodoContext>().expect("no ctx found");

    let toggle_edit = {
        let editing = editing.clone();
        Callback::from(move |_: MouseEvent| editing.set(!*editing))
    };

    // 内容更新（云端）
    let update_todo_param = use_state(String::new);
    let update_todo_content = {
        let id = todo.id;
        let param = update_todo_param.clone();
        let d = dispatcher.clone();
        use_async(async move {
            let result = todo_client().update_todo(id, UpdateTodoRequest {
                content: Some(param.to_string()),
                status: None,
            }).await.map_err(|e| e.to_string());

            d.dispatch(TodoAction::Refresh);

            result
        })
    };

    // 状态更新（云端）
    let update_status_param = use_state(|| Option::<TodoStatus>::None);
    let update_todo_status = {
        let id = todo.id;
        let param = update_status_param.clone();
        let d = dispatcher.clone();
        use_async(async move {
            let result = todo_client().update_todo(id, UpdateTodoRequest {
                content: None,
                status: *param,
            }).await;

            d.dispatch(TodoAction::Refresh);

            result.map_err(|e| e.to_string())
        })
    };

    let on_status = {
        let id = todo.id;
        let d = dispatcher.clone();
        let param = update_status_param.clone();
        let update_todo_status = update_todo_status.clone();
        let enable_remote = context.enable_remote;
        Callback::from(move |status: TodoStatus| {
            if enable_remote {
                param.set(Some(status));
                update_todo_status.run();
            } else {
                d.dispatch(TodoAction::Update(id, UpdateTodoRequest {
                    content: None,
                    status: Some(status),
                }))
            }
        })
    };

    let input_ref = use_node_ref();
    {
        let input_ref = input_ref.clone();
        use_effect_with(*editing, move |editing| {
            if *editing {
                if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                    let _ = input.focus();
                    input.select();
                }
            }
            || ()
        });
    }

    let handle_input = Rc::new({
        let d = dispatcher.clone();
        let id = todo.id;
        let editing = editing.clone();
        let update_todo_content = update_todo_content.clone();
        let param = update_todo_param.clone();
        let input_ref = input_ref.clone();
        let enable_remote = context.enable_remote;
        move || {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let value = input.value();
                let value = value.trim();
                if value.is_empty() {
                    return;
                }
                if enable_remote {
                    param.set(value.to_string());
                    update_todo_content.run();
                } else {
                    d.dispatch(TodoAction::Update(id, UpdateTodoRequest {
                        content: Some(value.to_string()),
                        status: None,
                    }));
                }
                editing.set(false);
            }
        }
    });

    let on_save_editing = {
        let handle_input = handle_input.clone();
        Callback::from(move |_: MouseEvent| handle_input())
    };

    let on_key = {
        let handle_input = handle_input.clone();
        let editing = editing.clone();
        Callback::from(move |e: KeyboardEvent| match e.key().as_str() {
            "Enter" => handle_input(),
            "Escape" => editing.set(false),
            _ => {}
        })
    };

    let status = todo.status;
    let checked = status == TodoStatus::Done;
    let checkable = status == TodoStatus::Todo || checked;
    let on_check = {
        let on_status = on_status.clone();
        Callback::from(move |_: MouseEvent| {
            on_status.emit(if checked { TodoStatus::Todo } else { TodoStatus::Done })
        })
    };

    html! {
        <li class={classes!("todo", status_class(status), editing.then_some("is-editing"))}>
            <span class="todo-index" aria-hidden="true">{ format!("{:02}", index) }</span>
            if checkable {
                <button type="button" role="checkbox"
                    class="check"
                    aria-checked={checked.to_string()}
                    aria-label={format!("Mark “{}” as {}", todo.content, if checked { "open" } else { "done" })}
                    onclick={on_check}>
                    <svg viewBox="0 0 20 20" aria-hidden="true" focusable="false">
                        <path class="check-mark" d="M5 10.5l3.2 3.2L15 6.8" />
                    </svg>
                </button>
            } else {
                <span class="todo-tag">{ status.to_string() }</span>
            }
            <div class="todo-content">
                if *editing {
                    <label class="sr-only" for={format!("edit-{}", todo.id)}>{ "Edit task" }</label>
                    <input class="todo-input" id={format!("edit-{}", todo.id)}
                        onkeydown={on_key}
                        type="text" ref={input_ref} value={todo.content.clone()}/>
                } else {
                    <span class="todo-text">{ todo.content.clone() }</span>
                }
            </div>
            <div class="todo-actions">
                <TodoControl
                    editing={*editing}
                    content={todo.content.clone()}
                    on_edit={toggle_edit}
                    on_save_editing={on_save_editing}
                    on_status={on_status}
                    status={status} />
            </div>
        </li>
    }
}
