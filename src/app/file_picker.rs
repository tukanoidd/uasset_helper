use std::path::PathBuf;

use crate::app::clickable_text::clickable_text_tooltip;
use iced::{button, tooltip, Alignment, Button, Color, Row, Text};
use rfd::AsyncFileDialog;

use super::GuiAppMessage;

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

pub fn widget<'state>(
    state: &'state mut button::State,
    asset: bool,
    button_text: &str,
    text: &str,
    tooltip: Option<String>,
) -> Row<'state, GuiAppMessage> {
    let text = clickable_text_tooltip(
        text,
        tooltip.map(|t| (t, tooltip::Position::Bottom, None)),
        None::<Color>,
    );

    Row::with_children(vec![
        Button::new(state, Text::new(button_text))
            .on_press(GuiAppMessage::OpenFilePicker(asset))
            .into(),
        text,
    ])
    .align_items(Alignment::Center)
    .spacing(10)
}
