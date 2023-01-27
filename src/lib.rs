use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::result::Result;

const INVALID_SIGNATURE: &str = "Invalid signature!";
const INVALID_FORMAT: &str = "Invalid file format!";
const UNEXPECTED_EOF: &str = "Unexpected end of file!";

#[derive(Debug, Clone)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn red(&self) -> u8 {
        self.red
    }

    pub fn green(&self) -> u8 {
        self.green
    }

    pub fn blue(&self) -> u8 {
        self.blue
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.red, self.green, self.blue)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((red, green, blue): (u8, u8, u8)) -> Self {
        Self::new(red, green, blue)
    }
}

impl From<Color> for (u8, u8, u8) {
    fn from(color: Color) -> Self {
        (color.red(), color.green(), color.blue())
    }
}

impl From<&Color> for (u8, u8, u8) {
    fn from(color: &Color) -> Self {
        (color.red(), color.green(), color.blue())
    }
}

#[derive(Debug, Clone)]
pub struct Ppm {
    width: usize,
    height: usize,
    color_depth: usize,
    pixels: Vec<Color>,
}

impl Ppm {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            color_depth: 255,
            pixels: vec![(0, 0, 0).into(); height * width],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = height;
    }

    pub fn color_depth(&self) -> usize {
        self.color_depth
    }

    pub fn set_color_depth(&mut self, color_depth: usize) {
        self.color_depth = color_depth;
    }

    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }

    pub fn iter_pixels(&self) -> impl Iterator<Item = (usize, usize, &Color)> {
        self.pixels
            .iter()
            .enumerate()
            .map(|(i, pixel)| (i % self.width, i / self.width, pixel))
    }

    pub fn set_pixels(&mut self, pixels: &[Color]) {
        self.pixels = pixels.to_vec();
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> Option<&Color> {
        self.pixels.get(y * self.width + x)
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), Error> {
        if let Some(pixel) = self.pixels.get_mut(y * self.width + x) {
            *pixel = color;
            Ok(())
        } else {
            Err(format!(
                "Pixel position ({},{}) is out of bounds for image size ({}, {})!",
                x, y, self.width, self.height
            )
            .into())
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let bytes = fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut size = (None, None);
        let mut color_depth = None;
        let mut i = 0;
        let mut next = 0;
        while let Some(pos) = bytes[i..].iter().position(|b| 0xA.eq(b)) {
            let chunk = &bytes[i..i + pos];
            i += pos + 1;
            if chunk.starts_with(&[0x23]) {
                continue;
            }
            match next {
                0 => {
                    if ![0x50, 0x36].eq(chunk) {
                        return Err(INVALID_SIGNATURE.into());
                    }
                    next += 1;
                }
                1 => {
                    let dimensions = String::from_utf8_lossy(chunk);
                    let (width, height) =
                        dimensions.split_once(0x20 as char).ok_or(UNEXPECTED_EOF)?;
                    let width = width.parse::<usize>().map_err(|_| INVALID_FORMAT)?;
                    let height = height.parse::<usize>().map_err(|_| INVALID_FORMAT)?;
                    size = (Some(width), Some(height));
                    next += 1;
                }
                2 => {
                    color_depth = Some(
                        String::from_utf8_lossy(chunk)
                            .parse::<usize>()
                            .map_err(|_| INVALID_FORMAT)?,
                    );
                    break;
                }
                _ => unreachable!(),
            }
        }
        if let ((Some(width), Some(height)), Some(color_depth)) = (size, color_depth) {
            let data = &bytes[i..];
            let mut ppm = Ppm::new(width, height);
            ppm.color_depth = color_depth;
            let mut y = 0;
            let mut x = 0;
            for i in 0..data.len() / 3 {
                if i > 0 && i % width == 0 {
                    y += 1;
                    x = 0;
                }
                let red = data[i * 3];
                let green = data[i * 3 + 1];
                let blue = data[i * 3 + 2];
                ppm.pixels[y * width + x] = Color::new(red, green, blue);
                x += 1;
            }
            Ok(ppm)
        } else {
            Err(INVALID_FORMAT.into())
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&[0x50, 0x36, 0xA]);
        bytes.extend_from_slice(format!("{} {}", self.width, self.height).as_bytes());
        bytes.extend_from_slice(&[0xA]);
        bytes.extend_from_slice(self.color_depth.to_string().as_bytes());
        bytes.extend_from_slice(&[0xA]);
        for y in 0..self.height {
            for x in 0..self.width {
                let (red, green, blue) = self.pixels[y * self.width + x].clone().into();
                bytes.extend_from_slice(&[red, green, blue]);
            }
        }
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self { message: err }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}
