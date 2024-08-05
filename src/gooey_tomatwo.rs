mod gooey;

use gooey::{Gooey, runtime_init};

fn main() -> Result<(), eframe::Error> {
    runtime_init();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "G 🍅 🍅 E Y   T 🍅 M A T W 🍅",
        options,
        Box::new(|cc| Box::new(Gooey::new(cc))),
    )
}