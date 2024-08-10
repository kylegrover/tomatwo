// tomatwo.rs

use structopt::StructOpt;
use std::path::PathBuf;
mod tomatwo_seed;
use tomatwo_seed::{Opt as LibOpt, process_video};

use std::io;
use std::io::ErrorKind;

#[derive(StructOpt, Debug)]
#[structopt(name = "tomato")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
    #[structopt(short, long, default_value = "void")]
    mode: String,
    #[structopt(short, default_value = "1")]
    countframes: usize,
    #[structopt(short, default_value = "1")]
    positframes: usize,
    #[structopt(short)]
    audio: bool,
    #[structopt(long)]
    firstframe: bool,
    #[structopt(short, default_value = "0.7")]
    kill: f32,
    #[structopt(short, default_value = "0.15")]
    kill_rel: f32,
    #[structopt(short, default_value = "1")]
    multiply: i32,
    #[structopt(short)]
    preview: bool
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    
    println!(r#"
    tomatwo - ufffd's rusty n dusty tomato fork
     _                        _        
    | |_ ___  _ __ ___   __ _| |_       _____  
    | __/   \| '_ ` _ \ / _` | __/\ /\ / /   \ 
    | || 🍅 |  | | | | | (_| | |_\ '  ' / 🍅 |
     \__\___/|_| |_| |_|\__,_|\__\\_/\_/ \___/ 
    v2.-09 last update 2024-08-08
    \\ Audio Video Interleave breaker
    
    glitch tool made with love for the glitch art community <3
    if you have any questions, would like to contact me
    or even hire me for performance / research / education
    you can shoot me an email at kaspar.ravel@gmail.com
    ___________________________________
    
    wb. https://www.kaspar.wtf 
    fb. https://www.facebook.com/kaspar.wtf 
    ig. https://www.instagram.com/kaspar.wtf 
    ___________________________________
    "#);

    let mut lib_opt = LibOpt {
        input: opt.input,
        mode: opt.mode,
        countframes: opt.countframes,
        positframes: opt.positframes,
        audio: opt.audio,
        firstframe: opt.firstframe,
        kill: opt.kill,
        kill_rel: opt.kill_rel,
        multiply: opt.multiply,
        preview: opt.preview
    };

    // check if input exists and is an avi file
    if !lib_opt.input.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Input file not found"));
    }
    if lib_opt.input.extension().unwrap_or_default() != "avi" {
        // default prep with ffmpeg
        println!("> Input file is not an avi file. Attempting to transcode with default ffmpeg settings...");
        // replace extension with avi
        let file_name = format!("{}.avi", lib_opt.input.file_stem().unwrap_or_default().to_str().unwrap_or_default());
        let ffmpeg = std::process::Command::new("ffmpeg")
            .args(&["-i", lib_opt.input.to_str().unwrap(), "-f", "avi", "-pix_fmt", "yuv420p", "-y", &file_name])
            .output()?;
        if !ffmpeg.status.success() {
            return Err(io::Error::new(ErrorKind::Other, "FFmpeg failed to transcode input file"));
        }
        println!("> Transcoding successful: {}", file_name);
        lib_opt.input = PathBuf::from(file_name);
    }


    println!("> Processing video...");
    let output_path = process_video(&lib_opt)?;

    Ok(())
}