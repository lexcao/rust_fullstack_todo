mod app;
mod state;
mod domain;
mod icon;

use app::*;

fn main() {
    yew::start_app::<App>();
}
