use eframe::egui;

pub fn render_welcome_screen(ui: &mut egui::Ui) {
    ui.add_space(20.0);
    ui.heading(egui::RichText::new("No video file selected").size(14.0));
    ui.add_space(10.0);
    ui.label("Click above and select an AVI or other video file to get started");
    ui.add_space(10.0);
    ui.label("• AVI files are used as is");
    ui.label("• Other video files will be converted to AVI using ffmpeg");
    ui.add_space(20.0);

    egui::CollapsingHeader::new("Software Requirements")
        .default_open(true)
        .show(ui, |ui| {
            ui.label("gooey tomatwo requires:");
            ui.indent("requirements", |ui| {
                ui.label("• ffmpeg - to prepare mp4s to avi and bake avis to mp4");
                ui.label("• ffplay - to preview videos");
            });
            ui.add_space(5.0);
            if ui.button("Download ffmpeg").clicked() {
                // Open URL: https://ffmpeg.org/download.html
            }
        });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        // ui.add(egui::Image::new(egui::include_image!("path/to/tomatwo_icon.png")));
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("About tomatwo").size(18.0).strong());
            ui.label("tomatwo is ufffd's rusty n dusty experimental fork of tomato.py, originally by Kaspar Ravel (MIT License)");
            if ui.link("https://github.com/itsKaspar/tomato").clicked() {
                // Open URL
            }
        });
    });

    ui.add_space(20.0);

    egui::CollapsingHeader::new("Datamosh Effects Guide")
        .default_open(false)
        .show(ui, |ui| {
            render_datamosh_guide(ui);
        });
}

pub fn render_datamosh_guide(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Modes (c - count, n - position):").strong());
    for (mode, description) in [
        ("void", "leaves frames in order while applying other parameters"),
        ("random", "randomizes frame order"),
        ("reverse", "reverse frame order"),
        ("invert", "flips each consecutive frame pair"),
        ("bloom", "duplicates c times p-frame number n"),
        ("pulse", "duplicates groups of c p-frames every n frames"),
        ("overlap", "copy group of c frames taken from every nth position"),
        ("jiggle", "take frame from around current position. n parameter is spread size [broken]"),
    ] {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(mode).monospace().strong());
            ui.label(description);
        });
    }

    ui.add_space(10.0);
    ui.label(egui::RichText::new("Other parameters:").strong());
    ui.label("• remove first frame (default on)");
    ui.label("• audio (not implemented yet)");
    ui.label("• kill: kill frames with too much data relative to the largest frame. default 0.7");
    ui.label("• kill_rel: kill frames with too much data relative to the previous frame size. default 0.15");

    ui.add_space(10.0);
    ui.label(egui::RichText::new("Examples:").strong());
    for (title, command) in [
        ("Take out all iframes:", "void: kill ~ 0.2 or kill_rel ~ 0.15"),
        ("Duplicate the 100th frame, 50 times:", "bloom c:50 n:100"),
        ("Duplicate every 10th frame 5 times each:", "pulse c:5 n:10"),
        ("Shuffle all frames in the video:", "random"),
        ("Copy 4 frames starting from every 2nd frame:", "overlap c:4 n:2"),
    ] {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(title).strong());
            ui.label(egui::RichText::new(command).monospace());
        });
    }
}