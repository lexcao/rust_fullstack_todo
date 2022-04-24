mod app;
mod state;
mod domain;
mod icon;
mod components;
mod hooks;
mod namespace;

use app::*;

fn main() {
    yew::start_app::<App>();
}
