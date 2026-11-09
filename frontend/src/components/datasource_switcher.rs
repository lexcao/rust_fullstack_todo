use web_sys::MouseEvent;
use yew::{Callback, classes, function_component, Html, html, Properties};

use crate::app::RemoteStatus;

#[derive(Properties, PartialEq, Clone)]
pub struct DataSourceSwitcherProps {
    pub enable_remote: bool,
    pub on_enable_remote: Callback<bool>,
    pub remote_status: RemoteStatus,
}

/// 存储位置切换：浏览器本地 / 云端。云端离线时禁用，并说明原因。
#[function_component(DataSourceSwitcher)]
pub fn data_source_switcher(DataSourceSwitcherProps { on_enable_remote, enable_remote, remote_status }: &DataSourceSwitcherProps) -> Html {
    let on_local = {
        let on_enable_remote = on_enable_remote.clone();
        Callback::from(move |_: MouseEvent| on_enable_remote.emit(false))
    };
    let remote_available = *remote_status == RemoteStatus::Online;
    let on_remote = {
        let on_enable_remote = on_enable_remote.clone();
        Callback::from(move |_: MouseEvent| {
            if remote_available {
                on_enable_remote.emit(true);
            }
        })
    };

    let (status_class, status_label) = match remote_status {
        RemoteStatus::Checking => ("is-checking", "checking"),
        RemoteStatus::Online => ("is-online", "online"),
        RemoteStatus::Offline => ("is-offline", "offline"),
    };
    let remote_title = match remote_status {
        RemoteStatus::Checking => "Checking the cloud API…",
        RemoteStatus::Online => "Save tasks on the actix-web server",
        RemoteStatus::Offline => "The cloud API is offline right now",
    };

    html! {
        <div class="switch" role="radiogroup" aria-label="Where tasks are saved">
            <button type="button" role="radio"
                class={classes!("switch-option", (!*enable_remote).then_some("is-active"))}
                aria-checked={(!*enable_remote).to_string()}
                title="Tasks are saved in this browser (localStorage)"
                onclick={on_local}>
                { "Browser" }
            </button>
            <button type="button" role="radio"
                class={classes!("switch-option", enable_remote.then_some("is-active"))}
                aria-checked={enable_remote.to_string()}
                aria-disabled={(!remote_available).to_string()}
                disabled={!remote_available}
                title={remote_title}
                onclick={on_remote}>
                <span class={classes!("status-dot", status_class)} aria-hidden="true"></span>
                { "Cloud" }
                <span class="sr-only">{ format!(" ({})", status_label) }</span>
            </button>
        </div>
    }
}
