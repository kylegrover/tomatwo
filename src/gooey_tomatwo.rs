mod gooey;

use crate::gooey::{Gooey};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "G 🍅 🍅 E Y   T 🍅 M A T W 🍅",
        options,
        Box::new(|cc| Box::new(Gooey::new(cc))),
    )
}