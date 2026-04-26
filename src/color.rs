#![allow(unused)]

pub trait GetEscape {
    fn get_bg(&self) -> String;
    fn get_fg(&self) -> String;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color256(u8);

impl Color256 {
    pub const NORMAL: Self = Self::new(7);

    pub const fn new(num: u8) -> Self {
        Self(num)
    }
}

impl GetEscape for Color256 {
    fn get_bg(&self) -> String {
        format!("\x1b[48;5;{}m", self.0)
    }

    fn get_fg(&self) -> String {
        format!("\x1b[38;5;{}m", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl GetEscape for Rgb {
    fn get_bg(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    fn get_fg(&self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }
}
