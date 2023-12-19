mod app;
mod states;
mod icons;
mod components;
mod hooks;
mod namespace;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
