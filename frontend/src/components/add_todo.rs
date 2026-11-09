use std::rc::Rc;

use web_sys::HtmlInputElement;
use yew::SubmitEvent;
use yew::{Callback, function_component, Html, html, Properties, use_context, use_node_ref, use_state, UseReducerDispatcher};
use yew_hooks::{use_async_with_options, UseAsyncOptions};

use common::model::CreateTodoRequest;

use crate::states::{TodoAction, TodoContext, TodoState};
use crate::app::todo_client;

#[derive(Properties, PartialEq, Clone)]
pub struct AddTodoProps {
    pub dispatcher: UseReducerDispatcher<TodoState>,
}

#[function_component(AddTodo)]
pub fn add_todo(AddTodoProps { dispatcher }: &AddTodoProps) -> Html {
    let input_ref = use_node_ref();
    let dispatcher = dispatcher.clone();

    let create_todo_param = use_state(String::new);
    let create_todo = {
        let d = dispatcher.clone();
        let param = create_todo_param.clone();
        use_async_with_options(async move {
            let content = param.to_string();
            let result = todo_client()
                .create_todo(CreateTodoRequest { content }).await
                .map_err(|e| e.to_string());

            d.dispatch(TodoAction::Refresh);

            result
        }, UseAsyncOptions { auto: false })
    };

    let context = use_context::<TodoContext>().expect("no ctx found");
    let handle_input = Rc::new({
        let input_ref = input_ref.clone();
        let param = create_todo_param.clone();
        let d = dispatcher.clone();
        let create_todo = create_todo.clone();

        move || {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let value = input.value();
                let value = value.trim();
                if value.is_empty() {
                    let _ = input.focus();
                    return;
                }
                if context.enable_remote {
                    param.set(value.to_string());
                    create_todo.run();
                } else {
                    d.dispatch(TodoAction::Add(value.to_string()));
                }
                input.set_value("");
                let _ = input.focus();
            }
        }
    });

    let on_submit = {
        let handle_input = handle_input.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            handle_input();
        })
    };

    html! {
        <form class="composer" onsubmit={on_submit}>
            <label class="sr-only" for="new-todo">{ "New task" }</label>
            <span class="composer-prompt" aria-hidden="true">{ "›" }</span>
            <input id="new-todo"
                    class="composer-input"
                    type="text"
                    ref={input_ref}
                    autocomplete="off"
                    maxlength="200"
                    placeholder={ "Add a task, press Enter" }/>
            <button class="btn btn-primary" type="submit">
                <span>{ "Add" }</span>
                <kbd class="kbd" aria-hidden="true">{ "↵" }</kbd>
            </button>
        </form>
    }
}
