//! WASM entry point: mount the Yew app into the page. Trunk builds this crate
//! for `wasm32-unknown-unknown` and injects the resulting module into the HTML.

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<beanifier_web::App>::new().render();
}
