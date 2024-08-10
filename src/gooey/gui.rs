use eframe::egui;
use std::sync::mpsc::{channel};
use std::thread;
use super::models::{Gooey, ProcessState, ProcessingStep};
use super::video_processing::{ffmpeg_to_avi, spawn_try_ffplay, ffmpeg_to_mp4};
use tomatwo_seed::{Opt, process_video, extract_frame_data};


impl Gooey {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn process_video(&self, preview: bool) {
        let tx = self.tx.clone();
        let avi_path = self.avi_path.clone().unwrap();
        let steps = self.processing_steps.clone();
    
        thread::spawn(move || {
            tx.send(ProcessState::Datamoshing).unwrap();
            let mut current_input = avi_path;
    
            for (i, step) in steps.iter().enumerate() {
                let opt = Opt {
                    input: current_input.clone(),
                    mode: step.mode.clone(),
                    countframes: step.count_frames,
                    positframes: step.posit_frames,
                    audio: false,
                    firstframe: false,
                    kill: step.kill,
                    kill_rel: step.kill_rel * step.kill_rel, // exp slider
                    multiply: step.multiply,
                    preview: preview && i == steps.len() - 1, // Only preview on the last step
                };
    
                match process_video(&opt) {
                    Ok(output_path) => {
                        if i == steps.len() - 1 {
                            if preview {
                                tx.send(ProcessState::Idle).unwrap();
                            } else {
                                tx.send(ProcessState::Done(output_path.clone())).unwrap();
                            }
                        }
                        current_input = output_path;
                    }
                    Err(e) => {
                        if preview && (e.kind() == std::io::ErrorKind::BrokenPipe) {
                            tx.send(ProcessState::Idle).unwrap();
                        } else {
                            eprintln!("Error processing video: {:?}", e);
                            tx.send(ProcessState::Error).unwrap();
                        }
                        return;
                    }
                }
            }
        });
    }

    fn extract_frame_data(&mut self) {
        if let Some(avi_path) = &self.avi_path {
            match extract_frame_data(avi_path) {
                Ok((frames, max_size)) => {
                    self.frame_data = Some((frames, max_size));
                },
                Err(e) => {
                    eprintln!("Error extracting frame data: {:?}", e);
                    self.frame_data = None;
                }
            }
        }
    }
}

impl eframe::App for Gooey {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // settings
        let dark_mode = egui::Visuals::dark();
        ctx.set_visuals(dark_mode);

        // logic
        if self.avi_path.is_some() {
            self.extract_frame_data();
        }

        // layout
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut button_label = "📁 select source video";
                let mut label = "Select an avi or other video file to get started";
                if let Some(path) = &self.input_path {
                    label = "🍅";
                    button_label = "🔄 change source video";
                }
                ui.label(label);
                
                ui.add_space(10.0);
                if ui.button(button_label).clicked() {
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
                    ui.label(format!("Selected: {}", path.file_name().unwrap().to_string_lossy()));
                }
            });
        });

        egui::SidePanel::left("steps_panel").show(ctx, |ui| {
            ui.heading("steps");
            ui.separator();

            for (index, step) in self.processing_steps.iter().enumerate() {
                let is_selected = self.selected_step == Some(index);
                if ui.selectable_label(is_selected, format!("Step {}: {}", index + 1, step.mode)).clicked() {
                    self.selected_step = Some(index);
                }
            }
            
            if ui.button("➕ add step").clicked() {
                self.processing_steps.push(ProcessingStep::default());
                self.selected_step = Some(self.processing_steps.len() - 1);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some((frame_data, max_frame_size)) = &self.frame_data {
                ui.label("Video Frame Data:");
                // Allocate a specific size for the visualization
                let available_width = ui.available_width();
                let viz_height = 100.0; // Adjust this value as needed
                let (response, painter) = ui.allocate_painter(egui::vec2(available_width, viz_height), egui::Sense::hover());
                let rect = response.rect;
            
                let bar_height = rect.height();
                let bar_width = rect.width() / frame_data.len() as f32;
            
                // Get the currently selected step, or use default values if no step is selected
                let current_step = self.selected_step.and_then(|index| self.processing_steps.get(index));
                let (kill, kill_rel) = current_step.map_or((1.0, 0.15), |step| (step.kill, step.kill_rel));
            
                for (i, frame) in frame_data.iter().enumerate() {
                    let x = rect.left() + i as f32 * bar_width;
                    let y = rect.bottom();
                    let height = (frame.size as f32 / *max_frame_size as f32) * rect.height();
                    
                    // Use the current step's kill and kill_rel values
                    let would_keep = frame.rel_size <= kill_rel + 1.0 && frame.size as f32 <= *max_frame_size as f32 * kill;
            
                    let color = if would_keep {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_rgb(255, 0, 0)
                    };
            
                    painter.rect_filled(
                        egui::Rect::from_min_max(egui::pos2(x, y - height), egui::pos2(x + bar_width, y)),
                        0.0,
                        color,
                    );
                }
                
                // Draw kill line using the current step's kill value
                painter.line_segment(
                    [
                        egui::pos2(rect.left(),  rect.top() + bar_height * (1.0 - kill)),
                        egui::pos2(rect.right(), rect.top() + bar_height * (1.0 - kill))
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(205, 155, 155))
                );
                
                ui.horizontal(|ui| {
                    ui.label("White: Frames that would be kept");
                    ui.label("Red: Frames that would be removed");
                });
                ui.horizontal(|ui| {
                    ui.label(format!("Total video frames: {}", frame_data.len()));
                
                    // Calculate frames kept using the current step's kill value
                    let frames_kept = frame_data.iter().filter(|frame| frame.rel_size <= kill_rel + 1.0 && frame.size as f32 <= *max_frame_size as f32 * kill).count();
                    ui.label(format!("Frames that would be kept: {}", frames_kept));
                });
            
                let hover_pos = ui.input(|i| i.pointer.hover_pos());
                if let Some(pos) = hover_pos {
                    if rect.contains(pos) {
                        let index = ((pos.x - rect.left()) / bar_width) as usize;
                        if index < frame_data.len() {
                            let frame = &frame_data[index];
                            let text = format!("Frame {}: Size {} bytes", index, frame.size);
                            painter.text(pos, egui::Align2::LEFT_BOTTOM, text, egui::TextStyle::Body.resolve(&ui.style()), ui.visuals().text_color());
                        }
                    }
                }
            
                // Display which step is being visualized
                if let Some(step_index) = self.selected_step {
                    ui.label(format!("Visualizing Step {}", step_index + 1));
                } else {
                    ui.label("No step selected. Showing default values.");
                }
            }

            if let Some(selected) = self.selected_step {
                ui.heading(format!("Edit Step {}", selected + 1));
                
                let mut remove_step = false;
                
                if let Some(step) = self.processing_steps.get_mut(selected) {
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("Mode")
                            .selected_text(&step.mode)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut step.mode, "void".to_string(), "Void");
                                ui.selectable_value(&mut step.mode, "random".to_string(), "Random");
                                ui.selectable_value(&mut step.mode, "reverse".to_string(), "Reverse");
                                ui.selectable_value(&mut step.mode, "invert".to_string(), "Invert");
                                ui.selectable_value(&mut step.mode, "bloom".to_string(), "Bloom");
                                ui.selectable_value(&mut step.mode, "pulse".to_string(), "Pulse");
                                ui.selectable_value(&mut step.mode, "jiggle".to_string(), "Jiggle");
                                ui.selectable_value(&mut step.mode, "overlap".to_string(), "Overlap");
                            });
                        if ui.button("🗑️").clicked() {
                            remove_step = true;
                        }
                    });

                    ui.add(egui::Slider::new(&mut step.count_frames, 1..=100).text("Count Frames"));
                    ui.add(egui::Slider::new(&mut step.posit_frames, 1..=100).text("Position Frames"));
                    ui.add(egui::Slider::new(&mut step.kill, 0.0..=1.0).text("Kill Threshold"));
                    ui.add(egui::Slider::new(&mut step.kill_rel, -0.1..=10.0).text("Kill Relative"));
                    ui.add(egui::Slider::new(&mut step.multiply, 1..=10).text("Multiply"));
                }

                if remove_step {
                    self.processing_steps.remove(selected);
                    self.selected_step = None;
                }
            }
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.avi_path.is_some() {
                    if ui.button("taste").clicked() { self.process_video(true); }
                    if ui.button("jar → avi)").clicked() { self.process_video(false); }
                }
            });

            match &self.process_state {
                ProcessState::Idle => {}
                ProcessState::Converting => { ui.add(egui::ProgressBar::new(0.5).text("Converting...")); }
                ProcessState::Datamoshing => { ui.add(egui::ProgressBar::new(0.5).text("Datamoshing...")); }
                ProcessState::Done(path) => {
                    ui.label(format!("Saved to: {}", path.display()));
                    if ui.button("play").clicked() {
                        let _handle = spawn_try_ffplay(path.clone());
                    }
                    if ui.button("bake → mp4").clicked() {
                        // prompt for file name/location then converts to mp4 and saves
                        let output = rfd::FileDialog::new().save_file();
                        if let Some(output) = output {
                            self.saved_path = Some(output.clone());
                            self.tx.send(ProcessState::Converting).unwrap();
                            match ffmpeg_to_mp4(&path, false) {
                                Ok(mp4_path) => {
                                    self.saved_path = Some(mp4_path);
                                    self.tx.send(ProcessState::Idle).unwrap();
                                },
                                Err(e) => {
                                    eprintln!("Error converting datamoshed video to MP4: {:?}", e);
                                    self.tx.send(ProcessState::Error).unwrap();
                                }
                            }
                        }
                    }
                    ui.label("Saves broken AVI into a reliable MP4");
                }
                ProcessState::Error => { ui.colored_label(egui::Color32::RED, "An error occurred."); }
            }
        });

        // Check for process state updates
        if let Ok(state) = self.rx.try_recv() {
            self.process_state = state;
            ctx.request_repaint();
        }
    }
}