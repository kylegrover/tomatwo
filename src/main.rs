use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use rfd;
use eframe;
use std::process::Command;

mod tomatwo_seed;
use tomatwo_seed::{Opt, process_video};

enum ProcessState {
    Idle,
    Converting,
    Datamoshing,
    Done,
}

struct MyApp {
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    mode: String,
    count_frames: usize,
    posit_frames: usize,
    kill: f32,
    process_state: ProcessState,
    preview_texture: Option<egui::TextureHandle>,
    rx: Receiver<ProcessState>,
    tx: Sender<ProcessState>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        Self {
            input_path: None,
            output_path: None,
            mode: "void".to_string(),
            count_frames: 1,
            posit_frames: 1,
            kill: 0.7,
            process_state: ProcessState::Idle,
            preview_texture: None,
            rx,
            tx,
        }
    }

    fn process_video(&self) {
        let opt = Opt {
            input: self.input_path.clone().unwrap(),
            mode: self.mode.clone(),
            countframes: self.count_frames,
            positframes: self.posit_frames,
            audio: false,
            firstframe: false,
            kill: self.kill,
        };

        let tx = self.tx.clone();
        
        thread::spawn(move || {
            tx.send(ProcessState::Converting).unwrap();
            let _avi_path = convert_mp4_to_avi(&opt.input);

            tx.send(ProcessState::Datamoshing).unwrap();
            match process_video(&opt) {
                Ok(_output_path) => {
                    tx.send(ProcessState::Done).unwrap();
                }
                Err(e) => {
                    eprintln!("Error processing video: {:?}", e);
                    tx.send(ProcessState::Idle).unwrap();
                }
            }
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Video Datamosher");

            if ui.button("Select MP4").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("MP4", &["mp4"]).pick_file() {
                    self.input_path = Some(path);
                }
            }

            if let Some(path) = &self.input_path {
                ui.label(format!("Selected file: {}", path.display()));
            }

            ui.horizontal(|ui| {
                ui.label("Mode:");
                egui::ComboBox::from_label("Mode")
                    .selected_text(&self.mode)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.mode, "void".to_string(), "Void");
                        ui.selectable_value(&mut self.mode, "random".to_string(), "Random");
                        ui.selectable_value(&mut self.mode, "reverse".to_string(), "Reverse");
                        ui.selectable_value(&mut self.mode, "bloom".to_string(), "Bloom");
                    });
            });

            ui.add(egui::Slider::new(&mut self.count_frames, 1..=100).text("Count Frames"));
            ui.add(egui::Slider::new(&mut self.posit_frames, 1..=100).text("Position Frames"));
            ui.add(egui::Slider::new(&mut self.kill, 0.0..=1.0).text("Kill Threshold"));

            if ui.button("Process Video").clicked() {
                self.process_video();
            }

            match self.process_state {
                ProcessState::Idle => {}
                ProcessState::Converting => { ui.label("Converting MP4 to AVI..."); }
                ProcessState::Datamoshing => { ui.label("Datamoshing..."); }
                ProcessState::Done => {
                    ui.label("Processing complete!");
                    if self.preview_texture.is_none() {
                        if let Some(output_path) = &self.output_path {
                            if let Ok(image) = load_first_frame(output_path) {
                                self.preview_texture = Some(ui.ctx().load_texture(
                                    "preview",
                                    image,
                                    egui::TextureOptions::default(),
                                ));
                            }
                        }
                    }
                }
            }
            if let Some(texture) = &self.preview_texture {
                ui.image(texture);
            }
        });

        // Check for process state updates
        if let Ok(state) = self.rx.try_recv() {
            self.process_state = state;
            ctx.request_repaint();
        }
    }
}


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Video Datamosher",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}

fn convert_mp4_to_avi(input: &PathBuf) -> Result<PathBuf, std::io::Error> {
    let output = input.with_extension("avi");
    let input_str = input.to_str().unwrap();
    let output_str = output.to_str().unwrap();

    let status = Command::new("ffmpeg")
        .args(&[
            "-i", input_str,
            "-c:v", "rawvideo",
            "-vf", "format=yuv420p",
            "-f", "avi",
            output_str
        ])
        .status()?;

    if status.success() {
        Ok(output)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "FFmpeg command failed"))
    }
}

fn load_first_frame(path: &PathBuf) -> Result<egui::ColorImage, std::io::Error> {
    let output = path.with_extension("png");
    let input_str = path.to_str().unwrap();
    let output_str = output.to_str().unwrap();

    let status = Command::new("ffmpeg")
        .args(&[
            "-i", input_str,
            "-vframes", "1",
            output_str
        ])
        .status()?;

    if status.success() {
        // Load the PNG file using image crate
        let img = image::open(&output).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        let pixels: Vec<u8> = rgb_img.into_raw();
        
        Ok(egui::ColorImage::from_rgb([width as usize, height as usize], &pixels))
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "FFmpeg command failed"))
    }
}