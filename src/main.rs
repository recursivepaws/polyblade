use cfg_if::cfg_if;
use dioxus::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[cfg(target_arch = "wasm32")]
use {
    log::info,
    polyblade::{graphics::WGPUInstance, render::driver::RenderDriver, Instant},
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

#[derive(Debug, Clone, EnumIter, PartialEq, Display)]
enum Platonic {
    Tetrahedron,
    Hexahedron,
    Octahedron,
    Dodecahedron,
    Icosahedron,
}

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

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div { class: "main-div",
            div { class: "menu-bar",
                div { class: "menu-group",
                    div { class: "menu-btn", "File" }
                    div { class: "dropdown",
                        div { class: "item",
                            "Open"
                            span { class: "shortcut", "#O" }
                        }

                        div { class: "item has-sub",
                            "Recent"
                            div { class: "submenu",
                                for preset in Platonic::iter() {
                                    div { class: "item", "file_{preset}.doc" }
                                }
                            }
                        }
                    }
                }
            }
            Outlet::<Route> {}
        }
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
                        let gpu = WGPUInstance::new(Canvas(el)).await;
                        info!("wgpu_instance created");
                        let (width, height) = (gpu.config.width, gpu.config.height);
                        let mut driver =
                            RenderDriver::new(&gpu.device, gpu.render_format, width, height);
                        loop {
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
