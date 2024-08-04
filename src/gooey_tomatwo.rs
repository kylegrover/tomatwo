use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use rfd;
use std::process::Command;
use tokio::process::Command as TokioCommand;
use tokio::task;

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
    saved_path: Option<PathBuf>,
    mode: String,
    count_frames: usize,
    posit_frames: usize,
    using_existing: bool,
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
            saved_path: None,
            mode: "void".to_string(),
            count_frames: 1,
            posit_frames: 1,
            using_existing: false,
            kill: 0.7,
            process_state: ProcessState::Idle,
            rx,
            tx,
        }
    }

    fn process_video(&self, preview: bool) {
        let opt = Opt {
            input: self.avi_path.clone().unwrap(),
            mode: self.mode.clone(),
            countframes: self.count_frames,
            positframes: self.posit_frames,
            audio: false,
            firstframe: false,
            kill: self.kill,
            preview,
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
                    match ffmpeg_to_avi(&path, false, &mut self.using_existing) {
                        Ok(avi_path) => {
                            self.avi_path = Some(avi_path);
                            self.tx.send(ProcessState::Idle).unwrap();
                        },
                        Err(e) => {
                            eprintln!("Error preparing input video as AVI: {:?}", e);
                            self.tx.send(ProcessState::Error).unwrap();
                        }
                    }
                    // let handle = spawn_ffmpeg_to_avi(path, false);
                }
            }

            if let Some(path) = &self.input_path {
                ui.label(format!("Selected file: {}", path.display()));
            }
            // if they didnt provide an avi, show the converted file we're using
            if self.avi_path.is_some() && !self.using_existing {
                ui.label("Converted video to AVI: ".to_owned() + self.avi_path.as_ref().unwrap().to_str().unwrap());
            }

            if self.using_existing {
                ui.label("Using existing AVI file: ".to_owned() + self.avi_path.as_ref().unwrap().to_str().unwrap());
                if ui.button("Re-convert video to AVI").clicked() {
                    self.tx.send(ProcessState::Converting).unwrap();
                    match ffmpeg_to_avi(&self.input_path.clone().unwrap(), true, &mut self.using_existing) {
                        Ok(avi_path) => {
                            self.avi_path = Some(avi_path);
                            self.using_existing = false;
                            self.tx.send(ProcessState::Idle).unwrap();
                        },
                        Err(e) => {
                            eprintln!("Error re-converting input video to AVI: {:?}", e);
                            self.tx.send(ProcessState::Error).unwrap();
                        }
                    }
                }
            }
            
                // if ui.button("Re-convert video to AVI").clicked() {
                //     self.tx.send(ProcessState::Converting).unwrap();
                //     match ffmpeg_to_avi(path, true) {
                //         Ok(avi_path) => {
                //             self.avi_path = Some(avi_path);
                //             self.tx.send(ProcessState::Idle).unwrap();
                //         },
                //         Err(e) => {
                //             eprintln!("Error re-converting input video to AVI: {:?}", e);
                //             self.tx.send(ProcessState::Error).unwrap();
                //         }
                //     }
                // }

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

            if self.avi_path.is_some() {
                if ui.button("Taste it").clicked()
                    { self.process_video(true); }
                if ui.button("Jar it").clicked()
                    { self.process_video(false); }
            }

            if self.saved_path.is_some() {
                ui.label("Saved to: ".to_owned() + self.saved_path.as_ref().unwrap().to_str().unwrap());
            }

            match &self.process_state {
                ProcessState::Idle => {}
                ProcessState::Converting => { ui.label("Converting Video to AVI..."); }
                ProcessState::Datamoshing => { ui.label("Datamoshing..."); }
                ProcessState::Done(path) => {
                    ui.label("Datamoshed video saved to:");
                    ui.label(path.to_str().unwrap());
                    if ui.button("Play Datamoshed Video").clicked() {
                        let handle = spawn_try_ffplay(path.clone());
                    }
                    
                    ui.horizontal(|ui| {
                        if ui.button("Bake output").clicked() {
                            if let Some(save_path) = rfd::FileDialog::new()
                                .add_filter("AVI", &["avi"])
                                .save_file() {
                                if let Err(e) = std::fs::copy(path, save_path) {
                                    eprintln!("Failed to save video: {:?}", e);
                                }
                            }
                        }
                        ui.label("Saves broken AVI into a reliable MP4");
                    });
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

fn ffmpeg_to_avi(input: &PathBuf, force: bool, &mut ref mut using_existing: &mut bool) -> Result<PathBuf, std::io::Error> {
    let output = input.with_extension("avi");
    // check if the output file already exists
    if !force && output.exists() {
        println!("Output file already exists: {:?} using that", output);
        *using_existing = true;
        return Ok(output);
    }

    let input_str = input.to_str().unwrap();
    let output_str = output.to_str().unwrap();

    let status = Command::new("ffmpeg")
        .args(&[
            "-i", input_str,
            "-c:v", "libxvid", // mjpeg, rawvideo
            "-pix_fmt", "yuv420p", // needed?
            "-q:v", "2", // 0?
            "-q:a", "0",
            "-y", // overwrite
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

async fn async_ffmpeg_to_mp4(input: PathBuf, fast: bool) -> Result<PathBuf, std::io::Error> {
    let output = input.with_extension("mp4");
    let input_str = input.to_str().unwrap();
    let output_str = output.to_str().unwrap();

    let status = TokioCommand::new("ffmpeg")
        .args(&[
            "-i", input_str,
            "-c:v", "libx264",
            "-f", "mp4",
            "-preset", if fast { "ultrafast" } else { "slow" },
            "-crf", if fast { "0" } else { "17" },
            "-pix_fmt", "yuv420p",
            output_str
        ])
        .status()
        .await?;

    if status.success() {
        Ok(output)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "FFmpeg command failed"))
    }
}

fn try_ffplay(path: &PathBuf) -> Result<(), std::io::Error> {
    Command::new("ffplay")
        .arg(path)
        .status()?;
    Ok(())
}
async fn async_try_ffplay(path: PathBuf) -> Result<(), std::io::Error> {
    TokioCommand::new("ffplay")
        .arg(path)
        .status()
        .await?;
    Ok(())
}


// spawners for async tasks
// pub fn spawn_ffmpeg_to_avi(input: PathBuf, force: bool) -> task::JoinHandle<Result<PathBuf, std::io::Error>> {
//     task::spawn(async move {
//         ffmpeg_to_avi(input, force).await
//     })
// }
pub fn spawn_ffmpeg_to_mp4(input: PathBuf, fast: bool) -> task::JoinHandle<Result<PathBuf, std::io::Error>> {
    task::spawn(async move {
        async_ffmpeg_to_mp4(input, fast).await
    })
}
pub fn spawn_try_ffplay(path: PathBuf) -> task::JoinHandle<Result<(), std::io::Error>> {
    task::spawn(async move {
        async_try_ffplay(path).await
    })
}