use std::path::PathBuf;

use iced::{button, container, tooltip, Alignment, Button, Color, Row, Text};
use rfd::AsyncFileDialog;

use crate::app::interactable_text::interactive_text_tooltip;

pub async fn open(asset: bool) -> Option<PathBuf> {
    let start_dir = dirs::home_dir().unwrap_or_default();

    match asset {
        true => {
            let file = AsyncFileDialog::new()
                .add_filter("uasset", &["uasset"])
                .add_filter("All", &["*"])
                .set_directory(start_dir)
                .pick_file()
                .await?;

            Some(file.path().to_path_buf())
        }
        false => {
            let folder = AsyncFileDialog::new()
                .set_directory(start_dir)
                .pick_folder()
                .await?;

            Some(folder.path().to_path_buf())
        }
    }
}

pub fn widget<'a, Message>(
    state: &'a mut button::State,
    button_text: &str,
    text: &str,
    tooltip: Option<String>,
    (text_on_press, text_on_shift_press): (Option<Message>, Option<Message>),
    button_on_press: Message,
    button_style: Option<impl Into<Box<dyn button::StyleSheet + 'a>>>,
    tooltip_style: Option<impl Into<Box<dyn container::StyleSheet + 'a>>>,
) -> Row<'a, Message>
where
    Message: Clone + 'a,
{
    let tooltip = tooltip.map(|t| (t, tooltip::Position::Bottom, None, tooltip_style));

    let text = interactive_text_tooltip(
        text,
        tooltip,
        None::<Color>,
        (text_on_press, text_on_shift_press),
        (None, None, None),
    );

    let mut button = Button::new(state, Text::new(button_text)).on_press(button_on_press);

    if let Some(button_style) = button_style {
        button = button.style(button_style);
    };

    Row::with_children(vec![button.into(), text])
        .align_items(Alignment::Center)
        .spacing(10)
}
