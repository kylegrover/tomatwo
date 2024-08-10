use std::path::PathBuf;

// from ../tomatwo_seed.rs
use tomatwo_seed::Frame;

pub enum ProcessState {
    Idle,
    Converting,
    Datamoshing,
    Done(PathBuf),
    Error,
}

pub struct Gooey {
    pub input_path: Option<PathBuf>,
    pub avi_path: Option<PathBuf>,
    pub saved_path: Option<PathBuf>,
    pub using_existing: bool,
    pub frame_data: Option<(Vec<Frame>, usize)>,
    pub process_state: ProcessState,
    pub rx: std::sync::mpsc::Receiver<ProcessState>,
    pub tx: std::sync::mpsc::Sender<ProcessState>,
    pub processing_steps: Vec<ProcessingStep>,
    pub selected_step: Option<usize>,
}

impl Default for Gooey {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Gooey {
            input_path: None,
            avi_path: None,
            saved_path: None,
            using_existing: false,
            frame_data: None,
            process_state: ProcessState::Idle,
            rx,
            tx,
            processing_steps: vec![ProcessingStep::default()],
            selected_step: None,
        }
    }
}

#[derive(Clone)]
pub struct ProcessingStep {
    pub mode: String,
    pub count_frames: usize,
    pub posit_frames: usize,
    pub kill: f32,
    pub kill_rel: f32,
    pub multiply: i32,
}

impl Default for ProcessingStep {
    fn default() -> Self {
        ProcessingStep {
            mode: "void".to_string(),
            count_frames: 30,
            posit_frames: 30,
            kill: 1.0,
            kill_rel: 0.15,
            multiply: 1,
        }
    }
}