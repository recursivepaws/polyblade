use cfg_if::cfg_if;
use dioxus::prelude::*;
use polyblade::render::message::{ConwayMessage, PolybladeMessage, PresetMessage, push_message};

mod components;
use components::MenuBar;

#[cfg(target_arch = "wasm32")]
use {
    log::info,
    polyblade::{Instant, graphics::WGPUInstance, render::driver::RenderDriver},
    wgpu::{SurfaceTarget::Canvas, TextureViewDescriptor},
};

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
use polyblade::native_paint::PolybladePaintSource;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const ERRORBG: Asset = asset!("/assets/errorbg.svg");

fn main() {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let _ = console_log::init();
        } else {
            colog::init();
        }
    }

    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

/// Shared navbar component. Also owns keyboard focus so shortcuts work anywhere.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            class: "main-div",
            tabindex: "0",
            autofocus: true,
            onmounted: move |evt| async move {
                let _ = evt.set_focus(true).await;
            },
            onkeydown: handle_key,
            div { class: "menu-bar", MenuBar {} }
            Outlet::<Route> {}
        }
    }
}

/// Shift+letter selects a preset; a bare letter triggers a Conway operation.
/// Case-insensitive.
fn handle_key(evt: Event<KeyboardData>) {
    use dioxus::html::Key;

    let Key::Character(ch) = evt.key() else {
        return;
    };
    let ch = ch.to_lowercase();

    let msg = if evt.modifiers().shift() {
        use PresetMessage::*;
        match ch.as_str() {
            "t" => Some(PolybladeMessage::Preset(Pyramid(3))),
            "c" => Some(PolybladeMessage::Preset(Prism(4))),
            "o" => Some(PolybladeMessage::Preset(Octahedron)),
            "d" => Some(PolybladeMessage::Preset(Dodecahedron)),
            "i" => Some(PolybladeMessage::Preset(Icosahedron)),
            _ => None,
        }
    } else {
        use ConwayMessage::*;
        match ch.as_str() {
            "d" => Some(PolybladeMessage::Conway(Dual)),
            "j" => Some(PolybladeMessage::Conway(Join)),
            "a" => Some(PolybladeMessage::Conway(Ambo)),
            "k" => Some(PolybladeMessage::Conway(Kis)),
            "t" => Some(PolybladeMessage::Conway(Truncate)),
            "e" => Some(PolybladeMessage::Conway(Expand)),
            "s" => Some(PolybladeMessage::Conway(Snub)),
            "b" => Some(PolybladeMessage::Conway(Bevel)),
            "c" => Some(PolybladeMessage::Conway(Chamfer)),
            _ => None,
        }
    };

    if let Some(msg) = msg {
        evt.prevent_default();
        push_message(msg);
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        PolyhedronCanvas {}
    }
}

#[component]
pub fn PolyhedronCanvas() -> Element {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use_effect(move || {
                if let Some(el) = polyblade::get_canvas("wgpu-canvas") {
                    spawn(async move {
                        let canvas = el.clone();
                        let mut gpu = WGPUInstance::new(Canvas(el)).await;
                        info!("wgpu_instance created");
                        let (width, height) = (gpu.config.width, gpu.config.height);
                        let mut driver =
                            RenderDriver::new(&gpu.device, gpu.render_format, width, height);
                        loop {
                            // Keep the backing store at the canvas's displayed
                            // size in physical pixels so nothing is stretched.
                            let dpr = web_sys::window().unwrap().device_pixel_ratio();
                            let width = (canvas.client_width() as f64 * dpr) as u32;
                            let height = (canvas.client_height() as f64 * dpr) as u32;
                            if width > 0
                                && height > 0
                                && (width, height) != (gpu.config.width, gpu.config.height)
                            {
                                canvas.set_width(width);
                                canvas.set_height(height);
                                gpu.config.width = width;
                                gpu.config.height = height;
                                gpu.surface.configure(&gpu.device, &gpu.config);
                                driver.resize(&gpu.device, width, height);
                            }

                            driver.tick(Instant::now());
                            let frame = gpu
                                .surface
                                .get_current_texture()
                                .expect("failed to acquire frame");
                            let view = frame.texture.create_view(&TextureViewDescriptor {
                                format: Some(gpu.render_format),
                                ..Default::default()
                            });
                            driver.draw(&gpu.device, &gpu.queue, &view);
                            frame.present();
                            polyblade::next_animation_frame().await;
                        }
                    });
                } else {
                    info!("failed to find canvas.")
                }
            });

            rsx! {
                div { class: "canvas-div",
                    canvas { id: "wgpu-canvas", width: 1000, height: 1000 }
                    img {
                        id: "error-background",
                        src: "{ERRORBG}",
                        object_fit: "contain",
                        background_color: "white",
                    }
                }
            }
        } else if #[cfg(feature = "native")] {
            let paint_id = dioxus_native::use_wgpu(PolybladePaintSource::new);

            // Blitz only repaints when the DOM changes, so tick a dummy
            // attribute at ~60fps to keep the animation running. Stopgap until
            // a proper redraw mechanism is exposed for custom paint sources.
            let mut frame = use_signal(|| 0u64);
            use_future(move || async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                    frame += 1;
                }
            });

            rsx! {
                div { class: "canvas-div",
                    canvas {
                        id: "wgpu-canvas",
                        "src": paint_id,
                        "data-frame": "{frame}",
                        width: 1000,
                        height: 1000,
                    }
                    img {
                        id: "error-background",
                        src: "{ERRORBG}",
                        object_fit: "contain",
                        background_color: "white",
                    }
                }
            }
        } else {
            // Server/fullstack builds have no GPU surface; render the static shell only.
            rsx! {
                div { class: "canvas-div",
                    canvas { id: "wgpu-canvas", width: 1000, height: 1000 }
                    img {
                        id: "error-background",
                        src: "{ERRORBG}",
                        object_fit: "contain",
                        background_color: "white",
                    }
                }
            }
        }
    }
}
