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
    pub mode: String,
    pub count_frames: usize,
    pub posit_frames: usize,
    pub using_existing: bool,
    pub kill: f32,
    pub kill_rel: f32,
    pub multiply: i32,
    pub frame_data: Option<(Vec<Frame>, usize)>,
    pub process_state: ProcessState,
    pub rx: std::sync::mpsc::Receiver<ProcessState>,
    pub tx: std::sync::mpsc::Sender<ProcessState>,
}