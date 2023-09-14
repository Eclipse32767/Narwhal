use iced::widget::{Column, Button, svg, Text};
use iced_style::theme;
use crate::{Message, clip_file_name, FONT_SIZE, IMAGE_SCALE};


#[derive(Clone)]
pub struct UIFile {//struct for holding the set of data needed for rendering
    pub name: String,
    pub original_index: usize,
    pub selected: bool,
    pub icon: String,
}

impl UIFile {
    pub async fn render<'a>(&self) -> Column<'a, Message> {//render self into a column
        let file_icon = self.icon.clone();
        let handle = svg::Handle::from_path(file_icon);
        let image = svg(handle).height(IMAGE_SCALE).width(IMAGE_SCALE);
        let text = Text::new(clip_file_name(self.name.clone())).size(FONT_SIZE);
        let button = if self.selected {
            Button::new(image).on_press(Message::FileClicked(self.original_index))
        } else {
            Button::new(image).on_press(Message::FileClicked(self.original_index)).style(theme::Button::Text)
        };
        Column::new().push(button).push(text).align_items(iced::Alignment::Center)
    }
}
/* 
pub fn localized_button<'a>(msg_id: &str, fontsize: u16) -> Button<'a, Message> {//create a button from gettext output with the fontsize indicated
    Button::new(Text::new(gettext(msg_id)).size(fontsize))
}
*/
pub fn string_button<'a>(msg: String, fontsize: u16)-> Button<'a, Message> {
    Button::new(Text::new(msg).size(fontsize))
}