use web_sys::MouseEvent;
use yew::{Callback, function_component, Html, html, Properties};
use yew_hooks::use_async;

use crate::icons;
use crate::app::todo_client;
use crate::hooks::use_retry;

#[derive(Properties, PartialEq, Clone)]
pub struct RemoteButtonProps {
    pub enable_remote: bool,
    pub on_enable_remote: Callback<bool>,
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
                    <icons::CloudOnline />
                } else {
                    <icons::CloudAlert />
                }
                </span>
            </button>
        </div>
    }
}


#[derive(Properties, PartialEq, Clone)]
pub struct DataSourceSwitcherProps {
    pub enable_remote: bool,
    pub on_enable_remote: Callback<bool>,
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
                    <span class="icon"><icons::CloudOffline /></span>
                </button>
            </div>
            <RemoteButton {on_enable_remote} enable_remote={*enable_remote} />
        </div>
    }
}
