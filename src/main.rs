use cfg_if::cfg_if;
use dioxus::prelude::*;
use polyblade::{graphics::Vertex, renderer::Triangle};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use ultraviolet::Vec3;

#[cfg(target_arch = "wasm32")]
use {
    log::info,
    polyblade::{graphics::WGPUInstance, renderer::Renderer},
    wgpu::SurfaceTarget::Canvas,
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
            console_log::init();
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
        SpinningCube {}
    }
}

fn triangle_model() -> Triangle {
    vec![
        Vertex {
            position: Vec3::new(0.0, 0.5, 0.0),
            color: Vec3::new(1.0, 0.0, 0.0),
        },
        Vertex {
            position: Vec3::new(-0.5, -0.5, 0.0),
            color: Vec3::new(0.0, 1.0, 0.0),
        },
        Vertex {
            position: Vec3::new(0.5, -0.5, 0.0),
            color: Vec3::new(0.0, 0.0, 1.0),
        },
    ]
}

#[component]
pub fn SpinningCube() -> Element {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let triangle = triangle_model();

            use_effect(move || {
                if let Some(el) = polyblade::get_canvas(&"wgpu-canvas") {
                    let tri = triangle.clone();

                    spawn(async move {
                        let gpu = WGPUInstance::new(Canvas(el)).await;
                        info!("wgpu_instance created");
                        let renderer = Renderer::new(&gpu.device, gpu.config.format, &tri);
                        renderer.render_surface(&gpu);
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
            let paint_id = dioxus_native::use_wgpu(move || {
                PolybladePaintSource::new(triangle_model())
            });

            rsx! {
                div { class: "canvas-div",
                    canvas { id: "wgpu-canvas", "src": paint_id, width: 1000, height: 1000 }
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
