use iced::{alignment::Vertical, Length};
use iced_aw::{menu::Item, menu_bar};
use iced_wgpu::Renderer;
use iced_widget::{button, column, container, row, text, Row, Space};
use iced_winit::{
    core::{Color, Element, Theme},
    runtime::{Program, Task},
};

use crate::render::{menu::MenuAble, message::*, state::AppState};

pub struct Controls {
    pub state: AppState,
}

impl Controls {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
        }
    }

    pub fn background_color(&self) -> Color {
        self.state.render.background_color
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Theme = Theme;
    type Message = PolybladeMessage;

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        message.process(&mut self.state)
    }

    fn view(&self) -> Element<Self::Message, Self::Theme, Self::Renderer> {
        let mut button_row = Row::new().spacing(10);
        for (i, color) in self
            .state
            .render
            .picker
            .palette
            .colors
            .iter()
            .cloned()
            .enumerate()
        {
            button_row = button_row.push(
                button("")
                    .style(move |theme, status| {
                        button::primary(theme, status).with_background(iced::Color::from(color))
                    })
                    .width(20)
                    .height(20)
                    .on_press(Self::Message::Render(RenderMessage::ColorPicker(
                        ColorPickerMessage::ChooseColor(i),
                    ))),
            );
        }

        let menu_bar = row![
            menu_bar!((PresetMessage::title(), PresetMessage::menu(&()))(
                ConwayMessage::title(),
                ConwayMessage::menu(&())
            )(
                RenderMessage::title(),
                RenderMessage::menu(&self.state.render)
            ))
            .style(iced_aw::menu::primary)
        ]
        .spacing(10.0);

        container(
            column![
                menu_bar.align_y(Vertical::Top),
                button_row,
                Space::new(Length::Fill, Length::Fill),
                self.state.model.polyhedron.svg(),
                Space::new(Length::Fill, Length::Fill),
                button(text(self.state.info.name())).on_press(self.state.info.wiki_message()),
                container(
                    row![
                        column![
                            text("Bowers:"),
                            text("Conway:"),
                            text("Faces:"),
                            text("Edges:"),
                            text("Vertices:"),
                        ],
                        column![
                            text(self.state.info.bowers()),
                            text(&self.state.info.conway),
                            text(self.state.info.faces),
                            text(self.state.info.edges),
                            text(self.state.info.vertices),
                        ]
                    ]
                    .spacing(20)
                    .align_y(Vertical::Bottom)
                )
                .style(|_| {
                    container::Style {
                        text_color: Some(iced::Color::BLACK),
                        ..Default::default()
                    }
                })
            ]
            .spacing(10),
        )
        .padding(10)
        .into()
    }
}
