#![allow(dead_code)]
#![deny(unsafe_code)]
use iced_style::{Color, button, Background};
use iced::theme::{self, Theme};

#[derive(Clone)]
pub struct ButtonStyle {
    pub border_radius: f32,
    pub txt_color: Color,
    pub bg_color: Option<Color>,
    pub border_color: Color,
    pub border_width: f32,
    pub shadow_offset: iced::Vector,
}

impl button::StyleSheet for ButtonStyle {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance { 
            shadow_offset: self.shadow_offset, 
            background: match self.bg_color {
                Some(x) => Some(Background::Color(x.clone())),
                None => None
            }, 
            border_radius: self.border_radius,
            border_width: self.border_width, 
            border_color: self.border_color.clone(), 
            text_color: self.txt_color.clone()
        }
    }
}

#[derive(Clone)]
pub struct ThemeSet {
    pub light: CustomTheme,
    pub dark: CustomTheme,
    pub custom: CustomTheme,
}

#[derive(Clone)]
pub struct CustomTheme {
    pub application: iced::theme::Palette,
    pub secondary: ButtonStyle,
    pub sidebar: ButtonStyle,
}

impl ButtonStyle {
    pub fn mk_theme(&self) -> theme::Button {
        theme::Button::Custom(std::boxed::Box::new(self.clone()))
    }
}
pub fn mk_app_theme(palette: iced::theme::Palette) -> iced::Theme {
    Theme::Custom(std::boxed::Box::new(iced::theme::Custom::new(palette)))
}