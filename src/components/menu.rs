use cfg_if::cfg_if;
use dioxus::prelude::*;
use polyblade::polyhedron::FaceTypeOption;
use polyblade::render::message::{
    ConwayMessage, PolybladeMessage, PresetMessage, RenderMessage, push_message,
    schlegel_face_options,
};
use strum::IntoEnumIterator;

/// The platonic solids expressible as presets, paired with their keyboard shortcuts.
fn platonic_items() -> Vec<(PresetMessage, &'static str)> {
    vec![
        (PresetMessage::Pyramid(3), "Shift+T"),
        (PresetMessage::Prism(4), "Shift+C"),
        (PresetMessage::Octahedron, "Shift+O"),
        (PresetMessage::Dodecahedron, "Shift+D"),
        (PresetMessage::Icosahedron, "Shift+I"),
    ]
}

fn conway_shortcut(op: &ConwayMessage) -> Option<&'static str> {
    use ConwayMessage::*;
    match op {
        Dual => Some("D"),
        Join => Some("J"),
        Ambo => Some("A"),
        Kis => Some("K"),
        Truncate => Some("T"),
        Expand => Some("E"),
        Snub => Some("S"),
        Bevel => Some("B"),
        Chamfer => Some("C"),
        SplitVertex(_) => None,
    }
}

#[component]
fn SizedPresetMenu(name: String, make: Callback<usize, PresetMessage>) -> Element {
    rsx! {
        div { class: "item has-sub",
            "{name}"
            div { class: "submenu",
                for n in 3..=8usize {
                    div {
                        class: "item",
                        onclick: move |_| push_message(PolybladeMessage::Preset(make(n))),
                        "{make(n)}"
                    }
                }
            }
        }
    }
}

/// Polls the backend-published Schlegel face-type options.
/// Renders one button per distinct type.
/// Only rendered when schlegel mode is on.
#[component]
fn SchlegelFaceMenu() -> Element {
    let mut options = use_signal(Vec::<FaceTypeOption>::new);

    use_future(move || async move {
        loop {
            options.set(schlegel_face_options());
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    polyblade::next_animation_frame().await;
                } else {
                    tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                }
            }
        }
    });

    rsx! {
        div { class: "menu-group top-right",
            div { class: "menu-btn", "Schlegel Face" }
            div { class: "dropdown",
                for option in options() {
                    div {
                        class: "item",
                        onclick: move |_| {
                            push_message(
                                PolybladeMessage::Render(
                                    RenderMessage::SchlegelFace(option.signature.clone()),
                                ),
                            );
                        },
                        "{option.label}"
                        span { class: "shortcut", "×{option.count}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn MenuBar(mut schlegel: Signal<bool>) -> Element {
    rsx! {
        div { class: "menu-group",
            div { class: "menu-btn", "Preset" }
            div { class: "dropdown",
                div { class: "item has-sub",
                    "Platonic solids"
                    div { class: "submenu",
                        for (preset, shortcut) in platonic_items() {
                            div {
                                class: "item",
                                onclick: move |_| { push_message(PolybladeMessage::Preset(preset.clone())) },
                                "{preset}"
                                span { class: "shortcut", "{shortcut}" }
                            }
                        }
                    }
                }
                SizedPresetMenu { name: "Prism", make: PresetMessage::Prism }
                SizedPresetMenu { name: "Antiprism", make: PresetMessage::AntiPrism }
                SizedPresetMenu { name: "Pyramid", make: PresetMessage::Pyramid }
            }
        }
        div { class: "menu-group",
            div { class: "menu-btn", "Conway" }
            div { class: "dropdown",
                for op in ConwayMessage::iter() {
                    div {
                        class: "item",
                        onclick: move |_| push_message(PolybladeMessage::Conway(op.clone())),
                        "{op}"
                        if let Some(key) = conway_shortcut(&op) {
                            span { class: "shortcut", "{key}" }
                        }
                    }
                }
            }
        }
        div { class: "menu-group",
            div { class: "menu-btn", "Render" }
            div { class: "dropdown",
                div {
                    class: "item",
                    onclick: move |_| {
                        let next = !schlegel();
                        schlegel.set(next);
                        push_message(PolybladeMessage::Render(RenderMessage::Schlegel(next)));
                    },
                    input { r#type: "checkbox", checked: schlegel() }
                    "Schlegel"
                }
            }
        }
        if schlegel() {
            SchlegelFaceMenu {}
        }
    }
}
