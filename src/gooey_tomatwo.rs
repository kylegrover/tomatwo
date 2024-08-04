use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use rfd;
use std::process::Command;

mod tomatwo_seed;
use tomatwo_seed::{Opt, process_video};

enum ProcessState {
    Idle,
    Converting,
    Datamoshing,
    Done(PathBuf),
    Error,
}

struct Gooey {
    input_path: Option<PathBuf>,
    avi_path: Option<PathBuf>,
    mode: String,
    count_frames: usize,
    posit_frames: usize,
    kill: f32,
    process_state: ProcessState,
    rx: Receiver<ProcessState>,
    tx: Sender<ProcessState>,
}

impl Gooey {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        Self {
            input_path: None,
            avi_path: None,
            mode: "void".to_string(),
            count_frames: 1,
            posit_frames: 1,
            kill: 0.7,
            process_state: ProcessState::Idle,
            rx,
            tx,
        }
    }

    fn process_video(&self) {
        let opt = Opt {
            input: self.avi_path.clone().unwrap(),
            mode: self.mode.clone(),
            countframes: self.count_frames,
            positframes: self.posit_frames,
            audio: false,
            firstframe: false,
            kill: self.kill,
        };

        let tx = self.tx.clone();
        
        thread::spawn(move || {
            tx.send(ProcessState::Datamoshing).unwrap();
            match process_video(&opt) {
                Ok(output_path) => {
                    tx.send(ProcessState::Done(output_path)).unwrap();
                }
                Err(e) => {
                    eprintln!("Error processing video: {:?}", e);
                    tx.send(ProcessState::Error).unwrap();
                }
            }
        });
    }
}

impl eframe::App for Gooey {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Video Datamosher");

            if ui.button("Select Source Video").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.input_path = Some(path.clone());
                    // check if its an AVI already
                    if path.extension().unwrap_or_default() == "avi" {
                        self.avi_path = Some(path);
                        return;
                    }
                    println!("Converting input video to AVI...");
                    self.tx.send(ProcessState::Converting).unwrap();
                    match ffmpeg_to_avi(&path) {
                        Ok(avi_path) => {
                            self.avi_path = Some(avi_path);
                            self.tx.send(ProcessState::Idle).unwrap();
                        },
                        Err(e) => {
                            eprintln!("Error preparing input video as AVI: {:?}", e);
                            self.tx.send(ProcessState::Error).unwrap();
                        }
                    }
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

            if ui.button("Process Video").clicked() && self.avi_path.is_some() {
                self.process_video();
            }

            match &self.process_state {
                ProcessState::Idle => {}
                ProcessState::Converting => { ui.label("Converting Video to AVI..."); }
                ProcessState::Datamoshing => { ui.label("Datamoshing..."); }
                ProcessState::Done(path) => {
                    ui.label("Processing complete!");
                    if ui.button("Play Datamoshed Video").clicked() {
                        // Open the video with ffplay command
                        if let Err(e) = Command::new("ffplay")
                            .arg(path)
                            .status() {
                            eprintln!("Failed to play video: {:?}", e);
                        }
                    }
                    if ui.button("Save Datamoshed Video").clicked() {
                        if let Some(save_path) = rfd::FileDialog::new()
                            .add_filter("AVI", &["avi"])
                            .save_file() {
                            if let Err(e) = std::fs::copy(path, save_path) {
                                eprintln!("Failed to save video: {:?}", e);
                            }
                        }
                    }
                }
                ProcessState::Error => { ui.label("An error occurred."); }
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
        "G 🍅 🍅 E Y   T 🍅 M A T W 🍅",
        options,
        Box::new(|cc| Box::new(Gooey::new(cc))),
    )
}

fn ffmpeg_to_avi(input: &PathBuf) -> Result<PathBuf, std::io::Error> {
    let output = input.with_extension("avi");
    // check if the output file already exists
    if output.exists() {
        println!("Output file already exists: {:?} using that", output);
        return Ok(output);
    }

    let input_str = input.to_str().unwrap();
    let output_str = output.to_str().unwrap();

    let status = Command::new("ffmpeg")
        // .args(&[
        //     "-i", input_str,
        //     "-c:v", "rawvideo",
        //     "-vf", "format=yuv420p",
        //     "-f", "avi",
        //     output_str
        // ])
        .args(&[
            "-i", input_str,
            "-c:v", "libxvid", // mjpeg?
            "-pix_fmt", "yuv420p", // needed?
            "-q:v", "2", // 0?
            "-q:a", "0",
            output_str
        ])
        .status()?;

    if status.success() {
        Ok(output)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "FFmpeg command failed"))
    }
}

fn ffmpeg_to_mp4(input: &PathBuf, fast: bool) -> Result<PathBuf, std::io::Error> {
    let output = input.with_extension("mp4");
    let input_str = input.to_str().unwrap();
    let output_str = output.to_str().unwrap();

    let status = Command::new("ffmpeg")
        .args(&[
            "-i", input_str,
            "-c:v", "libx264",
            "-f", "mp4",
            "-preset", if fast { "ultrafast" } else { "slow" },
            "-crf", if fast { "0" } else { "17" },
            "-pix_fmt", "yuv420p",
            output_str
        ])
        .status()?;

    if status.success() {
        Ok(output)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "FFmpeg command failed"))
    }
}