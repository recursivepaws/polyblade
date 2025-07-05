use cfg_if::cfg_if;
use dioxus::{html::g::fill, prelude::*};
use log::info;
use polyblade::{
    graphics::{Vertex, WGPUInstance},
    renderer::{Renderer, Triangle},
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use ultraviolet::Vec3;

#[cfg(target_arch = "wasm32")]
use wgpu::SurfaceTarget::Canvas;

// #[cfg(not(target_arch = "wasm32"))]

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

#[component]
pub fn SpinningCube() -> Element {
    let triangle: Triangle = vec![
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
    ];

    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        if let Some(el) = polyblade::get_canvas(&"wgpu-canvas") {
            let tri = triangle.clone();

            spawn(async move {
                let gpu = WGPUInstance::new(Canvas(el)).await;
                info!("wgpu_instance created");
                let renderer = Renderer::new(&gpu, &tri);
                renderer.render(&gpu);
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
}
