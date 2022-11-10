use std::borrow::Cow;
use std::path::PathBuf;

use iced::{
    widget::{tooltip, Row, Text},
    Alignment, Color, Renderer,
};
use iced_native::row;
use iced_native::widget::button;
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
    button_text: impl Into<Cow<'a, str>>,
    text: impl Into<Cow<'a, str>> + Clone,
    tooltip: Option<String>,
    (text_on_press, text_on_shift_press, text_on_ctrl_press): (
        Option<Message>,
        Option<Message>,
        Option<Message>,
    ),
    button_on_press: Message,
) -> Row<'a, Message>
where
    Message: Clone + 'a,
{
    let tooltip = tooltip.map(|t| (t, tooltip::Position::Bottom, None));

    let text = interactive_text_tooltip(
        text,
        tooltip,
        None::<Color>,
        (text_on_press, text_on_shift_press, text_on_ctrl_press),
        (None, None, None),
    );

    let button = button::<'a, Message, Renderer>(Text::new(button_text)).on_press(button_on_press);

    row![button, text]
        .align_items(Alignment::Center)
        .spacing(10)
}
