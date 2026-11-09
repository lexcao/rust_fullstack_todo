use gloo::timers::callback::Timeout;
use yew::{Callback, ContextProvider, function_component, Html, html, use_effect_with, use_reducer, use_state};
use yew_hooks::{use_async_with_options, UseAsyncOptions};

use common::client::{ScopeClient, TodoClient};
use common::model::{TodoResponse, TodoStatus};

use crate::brand::BRAND_NAME;
use crate::components::*;
use crate::states::{TodoAction, TodoContext, TodoState};
use crate::namespace;

pub fn todo_client() -> TodoClient {
    ScopeClient::default()
        .namespace(&namespace::get())
        .endpoint(
            option_env!("APP_REMOTE_ENDPOINT")
                .unwrap_or_else(|| "http://localhost:3000")
        )
        .todo_client()
}

/// 云端后端的可用状态：只在页面打开时探测一次，避免反复请求刷屏报错。
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RemoteStatus {
    Checking,
    Online,
    Offline,
}

/// 探测超过这个时间仍无响应，就按离线处理。
const PING_TIMEOUT_MS: u32 = 6_000;

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

    // 健康检查：挂载时自动跑一次。
    let ping = use_async_with_options(async move {
        todo_client().ping_client().ping().await.map_err(|e| e.to_string())
    }, UseAsyncOptions::enable_auto());

    let timed_out = use_state(|| false);
    {
        let timed_out = timed_out.clone();
        use_effect_with((), move |_| {
            let timeout = Timeout::new(PING_TIMEOUT_MS, move || timed_out.set(true));
            move || drop(timeout)
        });
    }

    let remote_status = match (&ping.data, &ping.error) {
        (Some(pong), _) if pong == "pong" => RemoteStatus::Online,
        (Some(_), _) | (_, Some(_)) => RemoteStatus::Offline,
        _ if *timed_out => RemoteStatus::Offline,
        _ => RemoteStatus::Checking,
    };

    let remotes = {
        let status_tab = status_tab.clone();
        use_async_with_options(async move {
            todo_client()
                .get_todos(*status_tab).await
                .map_err(|e| e.to_string())
        }, UseAsyncOptions { auto: false })
    };

    {
        let remotes = remotes.clone();
        let state = state.clone();
        let refresh = state.refresh;
        let enable_remote = context.enable_remote;
        use_effect_with((enable_remote, refresh), move |_| {
            if enable_remote {
                remotes.run();
            } else {
                state.save_to_local();
            }
            || ()
        });
    }

    let enable_remote = context.enable_remote;

    let data = if enable_remote {
        remotes.data.clone().unwrap_or_default()
    } else {
        state.locals
            .iter()
            .filter(|todo| status_tab.is_none_or(|tab| todo.status == tab))
            .cloned()
            .collect::<Vec<TodoResponse>>()
    };

    // 计数只在本地模式下可靠（云端只返回当前分组）。
    let counts = if enable_remote {
        None
    } else {
        Some(StatusCounts::from_todos(&state.locals))
    };

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
    let remote_error = if enable_remote { remotes.error.clone() } else { None };
    let loading_remote = enable_remote && remotes.loading && remotes.data.is_none();

    let todo_elements = data.iter()
        .enumerate()
        .map(|(index, todo)| html! {
            <TodoDetails key={todo.id} index={index + 1} todo={todo.clone()} dispatcher={state.dispatcher()}/>
        }).collect::<Html>();

    html! {
        <ContextProvider<TodoContext> context={(*context).clone()}>
            <div class="board" aria-label={format!("{} demo", BRAND_NAME)}>
                <div class="board-head">
                    <div>
                        <p class="board-kicker">{ "Live demo" }</p>
                        <h2 class="board-title">{ "Your list" }</h2>
                    </div>
                    <DataSourceSwitcher {on_enable_remote} {enable_remote} {remote_status} />
                </div>

                if let Some(counts) = counts.filter(|c| c.todo + c.done > 0) {
                    <Progress done={counts.done} total={counts.todo + counts.done} />
                }

                if remote_status == RemoteStatus::Offline && !enable_remote {
                    <DemoNotice />
                }

                <AddTodo dispatcher={state.dispatcher()} />

                <Tabs on_select={on_tab_select} selected={*status_tab} counts={counts} />

                <div class="board-body" aria-live="polite">
                    if let Some(error) = remote_error {
                        <p class="notice notice-error" role="alert">
                            { "Couldn't reach the cloud API. Switch back to Browser to keep going." }
                            <span class="notice-detail">{ error }</span>
                        </p>
                    } else if loading_remote {
                        <p class="board-loading">{ "Loading from the cloud…" }</p>
                    } else if empty_todos {
                        <EmptyState status={*status_tab} />
                    } else {
                        <ol class="todo-list">{ todo_elements }</ol>
                    }
                    if show_clear_deleted_button {
                        <ClearDeletedButton dispatcher={state.dispatcher()} />
                    }
                </div>
            </div>
        </ContextProvider<TodoContext>>
    }
}

#[function_component(DemoNotice)]
fn demo_notice() -> Html {
    html! {
        <div class="notice" role="status">
            <span class="notice-dot" aria-hidden="true"></span>
            <div>
                <p>{ "Demo mode — tasks are saved in this browser only. The cloud API is offline for now." }</p>
                <p lang="zh-CN" class="notice-zh">{ "演示模式：任务只保存在本浏览器，云端后端暂时离线。" }</p>
            </div>
        </div>
    }
}

#[derive(yew::Properties, PartialEq)]
struct EmptyStateProps {
    status: Option<TodoStatus>,
}

#[function_component(EmptyState)]
fn empty_state(EmptyStateProps { status }: &EmptyStateProps) -> Html {
    let (title, hint) = match status {
        None => ("Nothing on the list yet", "Type a task above and press Enter."),
        Some(TodoStatus::Todo) => ("All clear", "No open tasks. Add one above."),
        Some(TodoStatus::Done) => ("Nothing done yet", "Tick a task to see it here."),
        Some(TodoStatus::Archived) => ("No archived tasks", "Archive a task to tuck it away."),
        Some(TodoStatus::Deleted) => ("Bin is empty", "Archived tasks can be moved here."),
    };
    html! {
        <div class="empty">
            <svg class="empty-art" viewBox="0 0 120 72" aria-hidden="true" focusable="false">
                <rect x="8" y="10" width="104" height="14" rx="7" class="empty-bar" />
                <rect x="8" y="30" width="80" height="14" rx="7" class="empty-bar empty-bar-2" />
                <rect x="8" y="50" width="56" height="14" rx="7" class="empty-bar empty-bar-3" />
                <circle cx="100" cy="57" r="9" class="empty-dot" />
            </svg>
            <p class="empty-title">{ title }</p>
            <p class="empty-hint">{ hint }</p>
        </div>
    }
}

#[derive(yew::Properties, PartialEq)]
struct ProgressProps {
    done: usize,
    total: usize,
}

/// 进度条：只统计未归档的任务（Open + Done），数字全部来自本地真实数据。
#[function_component(Progress)]
fn progress(ProgressProps { done, total }: &ProgressProps) -> Html {
    let percent = if *total == 0 { 0 } else { done * 100 / total };
    let label = if *total == 0 {
        "No active tasks".to_string()
    } else {
        format!("{} of {} done", done, total)
    };
    html! {
        <div class="progress">
            <div class="progress-track" role="progressbar"
                aria-label="Tasks done"
                aria-valuemin="0" aria-valuemax="100" aria-valuenow={percent.to_string()}>
                <span class="progress-fill" style={format!("width:{}%", percent)}></span>
            </div>
            <span class="progress-label">{ label }</span>
        </div>
    }
}
