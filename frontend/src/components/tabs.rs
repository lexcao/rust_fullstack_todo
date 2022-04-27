use yew::{Callback, function_component, Html, html, Properties};

use common::model::TodoStatus;
use crate::icons;

#[derive(Properties, PartialEq, Clone)]
pub struct TabsProps {
   pub on_select: Callback<Option<TodoStatus>>,
   pub selected: Option<TodoStatus>,
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

