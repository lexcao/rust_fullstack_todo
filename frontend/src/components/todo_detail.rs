use std::rc::Rc;

use web_sys::{HtmlInputElement, KeyboardEvent, MouseEvent};
use yew::{Callback, function_component, Html, html, Properties, ToHtml, use_context, use_node_ref, use_state, UseReducerDispatcher};
use yew_hooks::use_async;

use common::model::{TodoResponse, TodoStatus, UpdateTodoRequest};
use crate::app::todo_client;

use crate::components::todo_control::*;
use crate::states::{TodoAction, TodoContext, TodoState};

#[derive(Properties, PartialEq, Clone)]
pub struct TodoDetailsProps {
    pub todo: TodoResponse,
    pub dispatcher: UseReducerDispatcher<TodoState>,
}

struct TodoStatusHtml(TodoStatus);

impl ToHtml for TodoStatusHtml {
    fn to_html(&self) -> Html {
        html! {self.0}
    }
}

#[function_component(TodoDetails)]
pub fn todo_details(TodoDetailsProps { todo, dispatcher }: &TodoDetailsProps) -> Html {
    let editing = use_state(|| false);

    let toggle_edit = {
        let editing = editing.clone();
        Callback::from(move |_| editing.set(!*editing))
    };

    let update_todo_param = use_state(|| String::new());
    let update_todo_content = {
        let id = todo.id.clone();
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

    let input_ref = use_node_ref();
    let context = use_context::<TodoContext>().expect("no ctx found");
    let handle_input = Rc::new({
        let d = dispatcher.clone();
        let id = todo.id.clone();
        let editing = editing.clone();
        let update_todo_content = update_todo_content.clone();
        let param = update_todo_param.clone();
        let input_ref = input_ref.clone();
        move || {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let value = input.value();
                let value = value.trim();
                if value.len() == 0 {
                    return;
                }
                if context.enable_remote {
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
        Callback::from(move |_: MouseEvent| {
            handle_input();
        })
    };

    let on_enter_press = {
        let handle_input = handle_input.clone();
        Callback::from(move |e: KeyboardEvent| if e.key() == "Enter" {
            handle_input();
        })
    };

    let status_tag_color = match todo.status {
        TodoStatus::Todo => { "is-info" }
        TodoStatus::Done => { "is-success" }
        TodoStatus::Archived => { "is-warning" }
        TodoStatus::Deleted => { "is-grey" }
    };

    html! {
        <div class="media is-align-items-center">
            <div class="media-left">
                <span class={format!("is-light is-rounded is-normal tag {}", status_tag_color)}>{ TodoStatusHtml(todo.status) }</span>
            </div>
            <div class="media-content">
                <div class="control">
                if *editing {
                    <input class="input"
                        onkeypress={on_enter_press}
                        readonly=false type="text" ref={input_ref} value={todo.content.clone()}/>
                } else {
                    <input class="input is-static"
                        onkeypress={on_enter_press}
                        readonly=true type="text" ref={input_ref} value={todo.content.clone()}/>
                }
                </div>
            </div>
            <div class="media-right">
                <TodoControl id={todo.id}
                    editing={*editing}
                    on_edit={toggle_edit}
                    on_save_editing={on_save_editing}
                    status={todo.status}
                    dispatcher={dispatcher.clone()} />
            </div>
        </div>
    }
}
