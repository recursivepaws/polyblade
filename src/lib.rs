pub mod graphics;

#[cfg(target_arch = "wasm32")]
use web_sys::{
    wasm_bindgen::JsCast,
    {window, HtmlCanvasElement},
};

#[cfg(target_arch = "wasm32")]
pub fn get_canvas(id: &str) -> Option<HtmlCanvasElement> {
    window().and_then(|win| win.document()).and_then(|doc| {
        doc.get_element_by_id(id)
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    })
}
