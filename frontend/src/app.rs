use yew::{Callback, ContextProvider, function_component, Html, html, use_effect_with_deps, use_reducer, use_state};
use yew_hooks::{use_async_with_options, UseAsyncOptions};

use common::client::{ScopeClient, TodoClient};
use common::model::TodoStatus;

use crate::components::*;
use crate::domain::Todo;
use crate::{icon, namespace};
use crate::state::{TodoAction, TodoContext, TodoState};

pub fn todo_client() -> TodoClient {
    ScopeClient::default()
        .namespace(&namespace::get())
        .endpoint(
            option_env!("APP_REMOTE_ENDPOINT")
                .unwrap_or_else(|| "http://localhost:3000")
        )
        .todo_client()
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
                            <a target="_black" href="https://github.com/lexcao/rust_fullstack_todo">
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

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <div class="container has-text-centered">
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
                {" Source code is available on "}
                <a target="_black" href="https://github.com/lexcao/rust_fullstack_todo">{ "GitHub" }</a>
            </p>
        </div>
    }
}
