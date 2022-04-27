use yew::{Html, Properties, function_component, use_context, use_state, UseReducerDispatcher, html, Callback};
use yew_hooks::use_async;
use common::model::TodoStatus;
use crate::states::{TodoAction, TodoContext, TodoState};
use crate::todo_client;

#[derive(Properties, PartialEq, Clone)]
pub struct ClearDeletedButtonProps {
    pub dispatcher: UseReducerDispatcher<TodoState>,
}

#[function_component(ClearDeletedButton)]
pub fn clear_deleted_button(ClearDeletedButtonProps { dispatcher }: &ClearDeletedButtonProps) -> Html {
    let confirm_status = use_state(|| false);

    let clear_deleted = {
        let d = dispatcher.clone();

        use_async(async move {
            let to_delete_ids = todo_client().get_todos(Some(TodoStatus::Deleted)).await
                .unwrap_or_else(|_| Vec::new())
                .into_iter()
                .map(|it| it.id)
                .collect::<Vec<i32>>();

            let result = todo_client().clear_todos(to_delete_ids).await
                .map(|_| "ok".to_string())
                .map_err(|e| e.to_string());

            d.dispatch(TodoAction::Refresh);

            result
        })
    };

    let context = use_context::<TodoContext>().expect("no ctx found");
    let on_click = {
        let confirm_status = confirm_status.clone();
        let d = dispatcher.clone();
        Callback::from(move |_| {
            if *confirm_status {
                if context.enable_remote {
                    clear_deleted.run();
                } else {
                    d.dispatch(TodoAction::ClearDeleted);
                }
                confirm_status.set(false);
            } else {
                confirm_status.set(true);
            }
        })
    };

    let cancel = {
        let confirm_status = confirm_status.clone();
        Callback::from(move |_| confirm_status.set(false))
    };

    let button_text = if *confirm_status {
        "Confirm to clear ?"
    } else {
        "Clear Deleted Todos"
    };

    html! {
        <div class="field is-grouped is-justify-content-center">
            <div class="control">
                <button onclick={on_click} class="button is-text is-fullwidth">{ button_text }</button>
            </div>
        if *confirm_status {
            <div class="control">
                <a onclick={cancel} class="button is-light">{ "Cancel" }</a>
            </div>
        }
        </div>
    }
}

