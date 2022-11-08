use iced::{
    keyboard,
    mouse::{self, Interaction},
    tooltip, Color, Element, Length, Point, Rectangle, Renderer, Text, Tooltip,
};
use iced_native::{
    event::Status,
    layout::{Limits, Node},
    renderer::Style,
    touch, Clipboard, Event, Layout, Shell, Widget,
};

use crate::app::GuiAppMessage;

pub struct InteractiveText<Message> {
    text: Text,

    on_press: Option<Message>,
    on_shift_press: Option<Message>,

    on_hover_in: Option<Message>,
    on_shift_hover: Option<Message>,
    on_hover_out: Option<Message>,

    shift_pressed: bool,
}

impl<Message> InteractiveText<Message> {
    pub fn new(text: Text) -> Self {
        Self {
            text,
            on_press: None,
            on_shift_press: None,

            on_hover_in: None,
            on_shift_hover: None,
            on_hover_out: None,

            shift_pressed: false,
        }
    }

    pub fn on_press(self, message: Message) -> Self {
        Self {
            on_press: Some(message),
            ..self
        }
    }

    pub fn on_shift_press(self, message: Message) -> Self {
        Self {
            on_shift_press: Some(message),
            ..self
        }
    }

    pub fn on_hover_in(self, message: Message) -> Self {
        Self {
            on_hover_in: Some(message),
            ..self
        }
    }

    pub fn on_shift_hover(self, message: Message) -> Self {
        Self {
            on_shift_hover: Some(message),
            ..self
        }
    }

    pub fn on_hover_out(self, message: Message) -> Self {
        Self {
            on_hover_out: Some(message),
            ..self
        }
    }
}

impl<Message> Widget<Message, Renderer> for InteractiveText<Message>
where
    Message: Clone,
{
    fn width(&self) -> Length {
        Widget::<Message, Renderer>::width(&self.text)
    }

    fn height(&self) -> Length {
        Widget::<Message, Renderer>::height(&self.text)
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        Widget::<Message, Renderer>::layout(&self.text, renderer, limits)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        Widget::<Message, Renderer>::draw(
            &self.text,
            renderer,
            style,
            layout,
            cursor_position,
            viewport,
        );
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> Status {
        let is_mouse_over = layout.bounds().contains(cursor_position);

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) if is_mouse_over => {
                    match (self.shift_pressed, &self.on_press, &self.on_shift_press) {
                        (true, _, Some(message)) => {
                            shell.publish(message.clone());

                            return Status::Captured;
                        }
                        (false, Some(message), _) => {
                            shell.publish(message.clone());

                            return Status::Captured;
                        }
                        _ => {}
                    }
                }
                mouse::Event::CursorEntered => {
                    if let Some(message) = &self.on_hover_in {
                        shell.publish(message.clone());

                        return Status::Captured;
                    }
                }
                mouse::Event::CursorLeft => {
                    if let Some(on_hover_out) = &self.on_hover_out {
                        shell.publish(on_hover_out.clone());

                        return Status::Captured;
                    }
                }
                _ => {}
            },
            Event::Touch(touch::Event::FingerPressed { position, .. })
                if layout.bounds().contains(position) =>
            {
                if let Some(message) = self.on_press.clone() {
                    shell.publish(message);

                    return Status::Captured;
                }
            }
            Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::ModifiersChanged(modifiers) => {
                    self.shift_pressed = modifiers.shift();

                    if is_mouse_over {
                        match self.shift_pressed {
                            true => {
                                if let Some(message) = &self.on_shift_hover {
                                    shell.publish(message.clone());

                                    return Status::Captured;
                                }
                            }
                            false => {
                                if let Some(message) = &self.on_hover_in {
                                    shell.publish(message.clone());

                                    return Status::Captured;
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }

        return Status::Ignored;
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> Interaction {
        let is_mouse_over = layout.bounds().contains(cursor_position);

        if is_mouse_over {
            Interaction::Pointer
        } else {
            Interaction::default()
        }
    }
}

impl<'a, Message> From<InteractiveText<Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(value: InteractiveText<Message>) -> Self {
        Element::new(value)
    }
}

pub fn interactive_text_tooltip<'a>(
    text: impl Into<String> + Clone,
    tooltip: Option<(String, tooltip::Position, Option<u16>)>,
    color: Option<impl Into<Color>>,
    (on_hover_in, on_shift_hover, on_hover_out): (
        Option<GuiAppMessage>,
        Option<GuiAppMessage>,
        Option<GuiAppMessage>,
    ),
) -> Element<'a, GuiAppMessage> {
    let mut text_widget = Text::new(text.clone()).size(16).width(Length::Shrink);

    if let Some(color) = color {
        text_widget = text_widget.color(color);
    }

    let mut text_widget =
        InteractiveText::new(text_widget).on_press(GuiAppMessage::SaveTextToClipboard(text.into()));

    if let Some(on_hover_in) = on_hover_in {
        text_widget = text_widget.on_hover_in(on_hover_in);
    }

    if let Some(on_shift_hover) = on_shift_hover {
        text_widget = text_widget.on_shift_hover(on_shift_hover);
    }

    if let Some(on_hover_out) = on_hover_out {
        text_widget = text_widget.on_hover_out(on_hover_out);
    }

    match tooltip {
        Some((tooltip, position, size)) => Tooltip::new(
            text_widget.on_shift_press(GuiAppMessage::SaveTextToClipboard(tooltip.clone())),
            tooltip,
            position,
        )
        .size(size.unwrap_or(16))
        .into(),
        None => text_widget.into(),
    }
}