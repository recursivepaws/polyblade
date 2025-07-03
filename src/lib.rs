use web_sys::{window, HtmlCanvasElement};
use web_sys::wasm_bindgen::JsCast;

pub mod graphics;

pub fn get_canvas(id: &str) -> Option<HtmlCanvasElement> {
    window().and_then(|win| win.document()).and_then(|doc| {
        doc.get_element_by_id(id)
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    })
}