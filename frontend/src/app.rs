use yew::prelude::*;
use web_sys::{HtmlInputElement};
use crate::domain::{Todo, TodoStatus};
use crate::state::{TodoAction, TodoState};

#[derive(Properties, PartialEq, Clone)]
pub struct TodoControlProps {
    id: usize,
    status: TodoStatus,
    dispatcher: UseReducerDispatcher<TodoState>,
}

#[function_component(TodoControl)]
pub fn todo_control(
    TodoControlProps { id, status, dispatcher }: &TodoControlProps
) -> Html {
    let status = status.clone();

    let update_status = {
        |status: TodoStatus| {
            let id = id.clone();
            let d = dispatcher.clone();
            Callback::from(move |_| { d.dispatch(TodoAction::UpdateStatus(id, status))})
        }
    };

    let toggle_status = match status {
        TodoStatus::Todo => TodoStatus::Done,
        TodoStatus::Done => TodoStatus::Todo,
        other => other,
    };

    let on_toggle_status = update_status(toggle_status);
    let on_archive = update_status(TodoStatus::Archived);
    let on_delete = update_status(TodoStatus::Deleted);

    let checked = status == TodoStatus::Done;
    let show_todo = status == TodoStatus::Todo || checked;

    let show_archive = show_todo;
    let show_delete = status == TodoStatus::Archived;

    html! {
        <div>
            if show_todo {
                <input type="checkbox" onclick={on_toggle_status} {checked} />
            }
            if show_archive {
                <button onclick={on_archive}>{ "Archive" }</button>
            }
            if show_delete {
                <button onclick={on_delete}>{ "Delete" }</button>
            }
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct TodoDetailsProps {
    todo: Todo,
    dispatcher: UseReducerDispatcher<TodoState>,
}

#[function_component(TodoDetails)]
fn todo_details(TodoDetailsProps { todo, dispatcher }: &TodoDetailsProps) -> Html {
    let editing = use_state(|| false);

    let toggle_edit = {
        let editing = editing.clone();
        Callback::from(move |_| editing.set(true))
    };

    let input_ref = use_node_ref();
    let on_save_editing = {
        let id = todo.id.clone();
        let input_ref = input_ref.clone();
        let editing = editing.clone();
        let d = dispatcher.clone();
        Callback::from(move |_|{
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                d.dispatch(TodoAction::Edit(id, input.value()));
                editing.set(false);
            }
        })
    };

    html! {
        <div>
            if *editing {
                <input ref={input_ref} value={todo.content.clone()}/>
                <button onclick={on_save_editing}>{ "Save" }</button>
            } else {
                <h3>
                    <span>{format!("[{:?}] - {} == ", todo.status, todo.id)}</span>{ todo.content.clone() }
                    <button onclick={toggle_edit}>{ "Edit" }</button>
                </h3>
            }
            <TodoControl id={todo.id} status={todo.status} dispatcher={dispatcher.clone()} />
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct AddTodoProps {
    dispatcher: UseReducerDispatcher<TodoState>,
}

#[function_component(AddTodo)]
fn add_todo(AddTodoProps { dispatcher }: &AddTodoProps) -> Html {
    let input_ref = use_node_ref();

    let on_submit = {
        let input_ref = input_ref.clone();
        let d = dispatcher.clone();
        Callback::from(move |_| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                d.dispatch(TodoAction::Add(input.value()));
                input.set_value("");
            }
        })
    };

    html! {
        <div>
            <input ref={input_ref} placeholder={"add new todo?"}/>
            <button onclick={on_submit}> {"ADD"}</button>
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_reducer(TodoState::default);

    use_effect_with_deps(
        move |state| {
            state.save_to_local();
            || ()
        },
        state.clone()
    );

    let todo_elements = state.todos.values()
        .filter(|todo| todo.status != TodoStatus::Archived)
        .filter(|todo| todo.status != TodoStatus::Deleted)
        .map(|todo| html! {
            <TodoDetails todo={todo.clone()} dispatcher={state.dispatcher()}/>
        }).collect::<Html>();

    let archived_todo_elements = state.todos.values()
        .filter(|todo| todo.status == TodoStatus::Archived)
        .map(|todo| html! {
            <TodoDetails todo={todo.clone()} dispatcher={state.dispatcher()}/>
        }).collect::<Html>();

    let deleted_todo_elements = state.todos.values()
        .filter(|todo| todo.status == TodoStatus::Deleted)
        .map(|todo| html! {
            <TodoDetails todo={todo.clone()} dispatcher={state.dispatcher()}/>
        }).collect::<Html>();

    html! {
        <main>
            <div>
                <h1>{ "Todos:" }</h1>
                <div>
                    { todo_elements }
                </div>
                <AddTodo dispatcher={state.dispatcher()} />
            </div>
            <div>
                <h1>{ "Archived:" }</h1>
                <div>
                    { archived_todo_elements }
                </div>
            </div>
            <div>
                <h1>{ "Deleted:" }</h1>
                <div>
                    { deleted_todo_elements }
                </div>
            </div>
        </main>
    }
}
