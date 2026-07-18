use dioxus::prelude::*;
use polyblade::render::message::{
    ConwayMessage, PolybladeMessage, PresetMessage, RenderMessage, push_message,
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

#[component]
pub fn MenuBar() -> Element {
    let mut schlegel = use_signal(|| false);

    rsx! {
        div { class: "menu-group",
            div { class: "menu-btn", "Preset" }
            div { class: "dropdown",
                div { class: "item has-sub",
                    "Platonic solids"
                    div { class: "submenu",
                        for (preset , shortcut) in platonic_items() {
                            div {
                                class: "item",
                                onclick: move |_| {
                                    push_message(PolybladeMessage::Preset(preset.clone()))
                                },
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
    }
}
