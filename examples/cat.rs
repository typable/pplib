use std::env;
use std::path::Path;

use pplib::Color;
use pplib::Ppm;

const HALF_BLOCK: &str = "▀";
const RESET: &str = "\x1b[0m";

trait AnsiEscapeCode {
    fn to_24bit_fg(&self) -> String;

    fn to_24bit_bg(&self) -> String;
}

impl AnsiEscapeCode for Color {
    fn to_24bit_fg(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.red(), self.green(), self.blue())
    }

    fn to_24bit_bg(&self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.red(), self.green(), self.blue())
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(path) => path,
        None => {
            println!("Argument is missing! Usage: cat <path>");
            return;
        }
    };
    if !Path::new(&path).exists() {
        println!("File doesn't exist! '{}'", path);
        return;
    }
    let ppm = match Ppm::from_file(&path) {
        Ok(ppm) => ppm,
        Err(err) => {
            println!("Unable to parse image! Cause: {}", err);
            return;
        }
    };
    render(&ppm);
}

fn render(ppm: &Ppm) {
    for y in (0..ppm.height()).step_by(2) {
        for x in 0..ppm.width() {
            if let Some(pixel) = ppm.pixel_at(x, y + 1) {
                print!("{}", pixel.to_24bit_bg());
            }
            if let Some(pixel) = ppm.pixel_at(x, y) {
                print!("{}{}", pixel.to_24bit_fg(), HALF_BLOCK);
            }
        }
        println!("{}", RESET);
    }
}
