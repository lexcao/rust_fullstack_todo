mod app;
mod state;
mod domain;

use app::*;

fn main() {
    yew::start_app::<App>();
}
