// tomatwo.rs

use structopt::StructOpt;
use std::path::PathBuf;
mod tomatwo_lib;
use tomatwo_lib::{Opt as LibOpt, process_video};

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
    v2.-1 last update 21.03.2020
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

    let lib_opt = LibOpt {
        input: opt.input,
        mode: opt.mode,
        countframes: opt.countframes,
        positframes: opt.positframes,
        audio: opt.audio,
        firstframe: opt.firstframe,
        kill: opt.kill,
    };

    let timer = std::time::Instant::now();

    println!("> Processing video...");
    let output_path = process_video(&lib_opt)?;
    println!("> Done! Output file: {:?}", output_path);
    println!("> Total time: {:.2?}", timer.elapsed());

    Ok(())
}