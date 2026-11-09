mod app;
mod brand;
mod states;
mod icons;
mod components;
mod namespace;

fn main() {
    let document = gloo::utils::document();
    match document.get_element_by_id("app") {
        Some(root) => {
            // 清掉静态的“加载中”占位，再挂载 Yew 应用。
            root.set_inner_html("");
            yew::Renderer::<app::App>::with_root(root).render();
        }
        None => {
            yew::Renderer::<app::App>::new().render();
        }
    }
}
