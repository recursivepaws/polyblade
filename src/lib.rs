pub mod graphics;
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub mod native_paint;
pub mod polyhedron;
pub mod render;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;
#[cfg(target_arch = "wasm32")]
pub use web_time::Instant;

#[cfg(target_arch = "wasm32")]
use web_sys::{
    wasm_bindgen::JsCast,
    {HtmlCanvasElement, window},
};

#[cfg(target_arch = "wasm32")]
pub fn get_canvas(id: &str) -> Option<HtmlCanvasElement> {
    window().and_then(|win| win.document()).and_then(|doc| {
        doc.get_element_by_id(id)
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    })
}

/// Resolves on the browser's next `requestAnimationFrame` callback.
#[cfg(target_arch = "wasm32")]
pub async fn next_animation_frame() {
    use std::{
        cell::RefCell,
        rc::Rc,
        task::{Poll, Waker},
    };
    use web_sys::wasm_bindgen::closure::Closure;

    struct RafState {
        fired: bool,
        waker: Option<Waker>,
    }
    let state = Rc::new(RefCell::new(RafState {
        fired: false,
        waker: None,
    }));

    let cb_state = state.clone();
    let closure = Closure::wrap(Box::new(move |_time: f64| {
        let mut state = cb_state.borrow_mut();
        state.fired = true;
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }) as Box<dyn FnMut(f64)>);

    window()
        .expect("no window")
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .expect("requestAnimationFrame failed");

    std::future::poll_fn(move |cx| {
        // Keep the closure alive until the callback has fired
        let _keep_alive = &closure;
        let mut state = state.borrow_mut();
        if state.fired {
            Poll::Ready(())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    })
    .await;
}
