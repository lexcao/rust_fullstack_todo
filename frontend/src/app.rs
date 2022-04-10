use yew::prelude::*;
use web_sys::{HtmlInputElement, Node};
use crate::domain::{Todo, TodoStatus};
use crate::state::{TodoAction, TodoState};
use crate::icon;

#[derive(Properties, PartialEq, Clone)]
pub struct TodoControlProps {
    id: u128,
    status: TodoStatus,
    editing: bool,
    on_edit: Callback<MouseEvent>,
    on_save_editing: Callback<MouseEvent>,
    dispatcher: UseReducerDispatcher<TodoState>,
}

#[function_component(TodoControl)]
pub fn todo_control(
    TodoControlProps { id, status, editing, on_edit, dispatcher, on_save_editing }: &TodoControlProps
) -> Html {
    let status = status.clone();
    let on_save_editing = on_save_editing.clone();
    let editing = *editing;

    let update_status = {
        |status: TodoStatus| {
            let id = id.clone();
            let d = dispatcher.clone();
            Callback::from(move |_| { d.dispatch(TodoAction::UpdateStatus(id, status)) })
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
        Callback::from(move |_| editing.set(!*editing))
    };

    let input_ref = use_node_ref();

    let handle_input = |input_ref: &NodeRef,
                        d: &UseReducerDispatcher<TodoState>,
                        editing: &UseStateHandle<bool>,
                        id: u128| {
        if let Some(input) = input_ref.cast::<HtmlInputElement>() {
            let value = input.value();
            if value == "" {
                return;
            }
            d.dispatch(TodoAction::Edit(id, value));
            editing.set(false);
        }
    };

    let on_save_editing = {
        let id = todo.id.clone();
        let input_ref = input_ref.clone();
        let editing = editing.clone();
        let d = dispatcher.clone();
        Callback::from(move |_: MouseEvent| {
            handle_input(&input_ref, &d, &editing, id);
        })
    };

    let on_enter_press = {
        let id = todo.id.clone();
        let input_ref = input_ref.clone();
        let editing = editing.clone();
        let d = dispatcher.clone();
        Callback::from(move |e: KeyboardEvent| if e.key() == "Enter" {
            handle_input(&input_ref, &d, &editing, id);
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
                <span class={format!("is-light is-rounded is-normal tag {}", status_tag_color)}>{ todo.status }</span>
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

#[derive(Properties, PartialEq, Clone)]
struct AddTodoProps {
    dispatcher: UseReducerDispatcher<TodoState>,
}

#[function_component(AddTodo)]
fn add_todo(AddTodoProps { dispatcher }: &AddTodoProps) -> Html {
    let input_ref = use_node_ref();

    let handle_input = |input_ref: &NodeRef, d: &UseReducerDispatcher<TodoState>| {
        if let Some(input) = input_ref.cast::<HtmlInputElement>() {
            let value = input.value();
            if value == "" {
                return;
            }
            d.dispatch(TodoAction::Add(value));
            input.set_value("");
        }
    };

    let on_submit = {
        let input_ref = input_ref.clone();
        let d = dispatcher.clone();
        Callback::from(move |_: MouseEvent| {
            handle_input(&input_ref, &d);
        })
    };

    let on_enter_press = {
        let input_ref = input_ref.clone();
        let d = dispatcher.clone();
        Callback::from(move |e: KeyboardEvent| if e.key() == "Enter" {
            handle_input(&input_ref, &d);
        })
    };

    html! {
        <div class="field has-addons">
            <div class="control is-expanded">
                <input class="input is-rounded is-info"
                        type="text"
                        ref={input_ref}
                        onkeypress={on_enter_press}
                        placeholder={ "Press 'enter' to submit" }/>
            </div>
            <div class="control">
                <button class="button is-rounded is-info" onclick={on_submit}>{ "Add" }</button>
            </div>
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_reducer(TodoState::default);
    let status_tab = use_state(|| Option::<TodoStatus>::None);

    use_effect_with_deps(
        move |state| {
            state.save_to_local();
            || ()
        },
        state.clone(),
    );

    let todo_data = state.todos.values()
        .rev()
        .filter(|todo| {
            status_tab.is_none() ||
                (status_tab.is_some() && todo.status == status_tab.unwrap())
        })
        .collect::<Vec<&Todo>>();

    let todo_elements = todo_data.iter()
        .map(|todo| html! {
            <TodoDetails todo={(*todo).clone()} dispatcher={state.dispatcher()}/>
        }).collect::<Html>();

    let on_tab_select = {
        let status_tab = status_tab.clone();
        Callback::from(move |status: Option<TodoStatus>| {
            status_tab.set(status)
        })
    };

    let empty_todos = todo_data.is_empty();
    let show_clear_deleted_button = *status_tab == Some(TodoStatus::Deleted) && !empty_todos;

    html! {
        <>
            <section class="hero is-link">
                <div class="hero-body">
                    <Header />
                </div>
                <div class="hero-foot">
                    <Tabs on_select={on_tab_select} selected={*status_tab} />
                </div>
            </section>
            <section class="container p-4 is-max-desktop">
                <AddTodo dispatcher={state.dispatcher()} />
                if show_clear_deleted_button {
                    <ClearDeletedButton dispatcher={state.dispatcher()} />
                }
                <div class="container" style="min-height:calc(100vh - 200px);overflow:visible">
                if empty_todos {
                    <p class="has-text-centered heading">{ "--- Empty ---" }</p>
                } else {
                    { todo_elements }
                }
                </div>
            </section>
            <section class="footer">
                <Footer />
            </section>
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct ClearDeletedButtonProps {
    dispatcher: UseReducerDispatcher<TodoState>,
}

#[function_component(ClearDeletedButton)]
pub fn clear_deleted_button(ClearDeletedButtonProps { dispatcher }: &ClearDeletedButtonProps) -> Html {
    let confirm_status = use_state(|| false);

    let on_click = {
        let confirm_status = confirm_status.clone();
        let d = dispatcher.clone();
        Callback::from(move |_| {
            if *confirm_status {
                d.dispatch(TodoAction::ClearDeleted);
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

#[derive(Properties, PartialEq, Clone)]
pub struct TabsProps {
    on_select: Callback<Option<TodoStatus>>,
    selected: Option<TodoStatus>,
}

#[function_component(Tabs)]
pub fn tabs(TabsProps { on_select, selected }: &TabsProps) -> Html {
    let status_tabs = [
        (Some(TodoStatus::Todo), html! { <icon::Edit /> }),
        (Some(TodoStatus::Done), html! { <icon::Check /> }),
        (None, html! { <icon::CheckAll /> }),
        (Some(TodoStatus::Archived), html! { <icon::Archive />}),
        (Some(TodoStatus::Deleted), html! { <icon::Delete />}),
    ].iter().map(|(status, icon): &(Option<TodoStatus>, Html)| {
        let active = if status == selected { "is-active" } else { "" };
        let status_name = match status {
            Some(value) => format!("{}", value),
            None => "All".to_owned(),
        };
        let on_tab_click = {
            let on_select = on_select.clone();
            let tab = status.clone();
            Callback::from(move |_| on_select.emit(tab))
        };

        html! {
            <li class={active} onclick={on_tab_click}>
                <a>
                    <span class="icon is-small">{ icon.clone() }</span>
                    <span>{ status_name }</span>
                </a>
            </li>
        }
    }).collect::<Html>();

    html! {
        <nav class="container is-max-desktop">
            <div class="tabs is-fullwidth is-boxed">
                <ul>
                    { status_tabs }
                </ul>
            </div>
        </nav>
    }
}

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <div class="container has-text-centered">
            <a target="_black" href="https://github.com/lexcao">
                <span class="icon"><icon::GitHub/></span>
            </a>
            <p class="title">{ "Todos" }</p>
            <p class="subtitle">
              { "🦀️ Rust Fullstack Application"}
            </p>
        </div>
    }
}

#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <div class="content has-text-centered">
            <p class="block">
               <strong>{ "🦀️ Rust Fullstack Todos Application" }</strong>
            </p>
            <p class="block">
                {"By "}
                <a target="_black" href="https://lexcao.io">{ "Lex Cao" }</a>
            </p>
            <p class="block">
                {" Source code is available "}
                <a target="_black" href="https://github.com/lexcao">{ "here" }</a>
            </p>
        </div>
    }
}