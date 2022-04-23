use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_hooks::{use_async, use_async_with_options, use_interval, UseAsyncOptions};

use common::client::{ScopeClient, TodoClient};
use common::model::{CreateTodoRequest, TodoStatus, UpdateTodoRequest};

use crate::domain::Todo;
use crate::icon;
use crate::state::{TodoAction, TodoContext, TodoState};

#[derive(Properties, PartialEq, Clone)]
pub struct TodoControlProps {
    id: i32,
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

    let update_todo_param = use_state(|| String::new());
    let update_todo_content = {
        let id = todo.id.clone() as i32;
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
    let dispatcher = dispatcher.clone();

    let create_todo_param = use_state(|| String::new());
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
        let context = context.clone();
        let create_todo = create_todo.clone();

        move || {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let value = input.value();
                let value = value.trim();
                if value.len() == 0 {
                    return;
                }
                if context.enable_remote {
                    param.set(value.to_string());
                    create_todo.run();
                } else {
                    d.dispatch(TodoAction::Add(value.to_string()));
                }
                input.set_value("");
            }
        }
    });

    let on_submit = {
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

fn todo_client() -> TodoClient {
    ScopeClient::default()
        .namespace("test")
        .endpoint("http://localhost:3000")
        .todo_client()
}

#[derive(Properties, PartialEq, Clone)]
pub struct RemoteButtonProps {
    enable_remote: bool,
    on_enable_remote: Callback<bool>,
}

#[function_component(RemoteButton)]
pub fn remote_button(RemoteButtonProps { enable_remote, on_enable_remote }: &RemoteButtonProps) -> Html {
    let health_check = use_async(async move {
        let client = todo_client().ping_client();

        client.ping().await.map_err(|e| e.to_string())
    });

    let remote_available = match &health_check.data {
        None => false,
        Some(response) => response == "pong",
    };

    let retrying = {
        let heath_check = health_check.clone();
        let on_retry = move || { !remote_available };
        use_retry(move || {
            heath_check.run();
        }, on_retry, 2)
    };

    let on_remote = {
        let on_enable_remote = on_enable_remote.clone();
        Callback::from(move |_: MouseEvent| {
            if !remote_available {
                return;
            }
            on_enable_remote.emit(true);
        })
    };

    let button = "button is-rounded".to_string();
    let remote_class = if *enable_remote {
        "is-success"
    } else if retrying {
        "is-loading"
    } else if remote_available {
        "is-success is-light"
    } else { "is-danger is-light" };

    let remote_class = format!("{} {}", button, remote_class);
    let remote_tooltip = if remote_available { "data is saved on remote server" } else { "remote is not available" };

    html! {
        <div class="control" data-tooltip={remote_tooltip}>
            <button class={remote_class} onclick={on_remote} disabled={!remote_available}>
                <span class="icon">
                if remote_available {
                    <icon::CloudOnline />
                } else {
                    <icon::CloudAlert />
                }
                </span>
            </button>
        </div>
    }
}

fn use_retry<Callback, Condition>(callback: Callback, on_retry: Condition, times: u32) -> bool
    where
        Callback: Fn() + 'static,
        Condition: Fn() -> bool + 'static,
{
    let interval = use_state(|| 200);
    let interval_mut = interval.clone();
    let retry = use_state(|| 0);
    use_interval(move || {
        callback();
        if !on_retry() {
            interval_mut.set(0);
            return;
        }
        interval_mut.set(*interval_mut * 2);
        if *retry == times {
            interval_mut.set(0);
        } else {
            retry.set(*retry + 1);
        }
    }, *interval);

    return *interval != 0;
}

#[derive(Properties, PartialEq, Clone)]
pub struct DataSourceSwitcherProps {
    enable_remote: bool,
    on_enable_remote: Callback<bool>,
}

#[function_component(DataSourceSwitcher)]
pub fn data_source_switcher(DataSourceSwitcherProps { on_enable_remote, enable_remote }: &DataSourceSwitcherProps) -> Html {
    let on_local = {
        let on_enable_remote = on_enable_remote.clone();
        Callback::from(move |_: MouseEvent| {
            on_enable_remote.emit(false);
        })
    };

    let button = "button is-rounded".to_string();
    let local_class = if *enable_remote { "is-light" } else { "is-info" };
    let local_class = format!("{} {}", button, local_class);

    html! {
         <div class="field has-addons is-justify-content-center">
            <div class="control" data-tooltip="data is saved in the browser">
                <button class={local_class} onclick={on_local} >
                    <span class="icon"><icon::CloudOffline /></span>
                </button>
            </div>
            <RemoteButton {on_enable_remote} enable_remote={*enable_remote} />
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_reducer(TodoState::default);
    let status_tab = use_state(|| Option::<TodoStatus>::None);

    let context = use_state(|| TodoContext {
        enable_remote: false,
    });

    let on_enable_remote = {
        let context = context.clone();
        Callback::from(move |value: bool| {
            context.set(TodoContext { enable_remote: value })
        })
    };

    let remotes = {
        let status_tab = status_tab.clone();
        use_async_with_options(async move {
            todo_client()
                .get_todos(*status_tab).await
                .map_err(|e| e.to_string())
                .map(|it|
                    it.into_iter()
                        .map(|it| Todo {
                            status: it.status,
                            content: it.content,
                            id: it.id as i32,
                        })
                        .collect::<Vec<Todo>>()
                )
        }, UseAsyncOptions { auto: false })
    };

    {
        let remotes = remotes.clone();
        let state = state.clone();
        let refresh = state.refresh;
        let enable_remote = context.clone().enable_remote;
        use_effect_with_deps(
            move |_| {
                if enable_remote {
                    remotes.run();
                } else {
                    state.save_to_local();
                }
                || ()
            },
            (enable_remote, refresh),
        );
    }

    let data = {
        let enable_remote = context.clone().enable_remote;
        if enable_remote {
            match remotes.data.clone() {
                Some(data) => data,
                None => vec![],
            }
        } else {
            state.locals
                .iter()
                .cloned()
                .filter(|todo| {
                    status_tab.is_none() ||
                        (status_tab.is_some() && todo.status == status_tab.unwrap())
                })
                .collect::<Vec<Todo>>()
        }
    };

    let todo_elements = data.iter()
        .map(|todo| html! {
            <TodoDetails todo={(*todo).clone()} dispatcher={state.dispatcher()}/>
        }).collect::<Html>();

    let on_tab_select = {
        let status_tab = status_tab.clone();
        let state = state.clone();
        Callback::from(move |status: Option<TodoStatus>| {
            if status == *status_tab {
                return;
            }
            status_tab.set(status);
            state.dispatch(TodoAction::Refresh);
        })
    };

    let empty_todos = data.is_empty();
    let show_clear_deleted_button = *status_tab == Some(TodoStatus::Deleted) && !empty_todos;

    html! {
        <ContextProvider<TodoContext> context={(*context).clone()}>
            <section class="hero is-link">
                <div class="hero-head">
                    <div class="container p-4 is-max-desktop">
                        <div class="is-justify-content-flex-end is-flex">
                            <a target="_black" href="https://github.com/lexcao">
                                <span class="icon"><icon::GitHub/></span>
                            </a>
                        </div>
                    </div>
                </div>
                <div class="hero-body">
                    <Header/>
                </div>
                <div class="hero-foot">
                    <Tabs on_select={on_tab_select} selected={*status_tab} />
                </div>
            </section>
            <section class="container p-4 is-max-desktop">
                <div class="is-flex is-justify-content-space-between">
                    <DataSourceSwitcher {on_enable_remote}
                            enable_remote={context.enable_remote}/>
                    <div class="is-flex-grow-1 pl-4">
                        <AddTodo dispatcher={state.dispatcher()} />
                    </div>
                </div>
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
        </ContextProvider<TodoContext>>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct ClearDeletedButtonProps {
    dispatcher: UseReducerDispatcher<TodoState>,
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

#[derive(Properties, PartialEq, Clone)]
pub struct HeaderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    html! {
        <div class="container has-text-centered">
            { for props.children.iter() }
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
