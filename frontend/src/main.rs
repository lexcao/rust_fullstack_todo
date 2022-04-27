mod app;
mod states;
mod domain;
mod icons;
mod components;
mod hooks;
mod namespace;

use app::*;

fn main() {
    yew::start_app::<App>();
}
