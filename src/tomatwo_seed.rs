// tomatwo_lib.rs

use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use rand::seq::SliceRandom;
use memmap2::{Mmap, MmapOptions};
use rayon::prelude::*;
use tempfile;
use rand::Rng;

#[derive(Clone, Debug)]
pub struct Opt {
    pub input: PathBuf,
    pub mode: String,
    pub countframes: usize,
    pub positframes: usize,
    pub audio: bool,
    pub firstframe: bool,
    pub kill: f32,
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub offset: usize,
    pub size: usize,
    pub frame_type: FrameType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrameType {
    Video,
    Audio,
    Void,
}

pub fn bstream_until_marker(input: &PathBuf, output: &PathBuf, marker: Option<&[u8]>, startpos: usize) -> io::Result<usize> {
    let input_file = File::open(input)?;
    let mut output_file = File::create(output)?;
    let mmap = unsafe { MmapOptions::new().offset(startpos as u64).map(&input_file)? };

    if let Some(marker) = marker {
        if let Some(pos) = mmap.windows(marker.len()).position(|window| window == marker) {
            output_file.write_all(&mmap[..pos])?;
            return Ok(startpos + pos);
        }
    }

    output_file.write_all(&mmap)?;
    Ok(startpos + mmap.len())
}

pub fn build_frame_table(temp_movi: &PathBuf, include_audio: bool) -> io::Result<Vec<Frame>> {
    let file = File::open(temp_movi)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let mut frame_table: Vec<Frame> = mmap.par_windows(4)
        .enumerate()
        .filter_map(|(i, window)| {
            match window {
                b"00dc" => Some(Frame { offset: i, size: 0, frame_type: FrameType::Video }),
                b"01wb" if include_audio => Some(Frame { offset: i, size: 0, frame_type: FrameType::Audio }),
                _ => None,
            }
        })
        .collect();

    for i in 0..frame_table.len() {
        frame_table[i].size = if i + 1 < frame_table.len() {
            frame_table[i + 1].offset - frame_table[i].offset
        } else {
            mmap.len() - frame_table[i].offset
        };
    }

    Ok(frame_table)
}

pub fn process_frames(clean_frames: &[Frame], opt: &Opt) -> Vec<Frame> {
    match opt.mode.as_str() {
        "void" => clean_frames.to_vec(),
        "random" => {
            let mut rng = rand::thread_rng();
            let mut frames = clean_frames.to_vec();
            frames.shuffle(&mut rng);
            frames
        },
        "reverse" => clean_frames.iter().rev().cloned().collect(),
        "invert" => clean_frames.chunks(2).flat_map(|chunk| chunk.iter().rev()).cloned().collect(),
        "bloom" => {
            let repeat = opt.countframes;
            let frame = opt.positframes;
            let (lista, listb) = clean_frames.split_at(frame);
            [lista, &vec![clean_frames[frame].clone(); repeat], listb].concat()
        },
        "pulse" => {
            let pulse_len = opt.countframes;
            let pulse_ryt = opt.positframes;
            clean_frames.iter().enumerate().flat_map(|(j, frame)| {
                if j % pulse_ryt == 0 {
                    vec![frame.clone(); pulse_len]
                } else {
                    vec![frame.clone()]
                }
            }).collect()
        },
        "jiggle" => {
            let amount = opt.positframes as f64;
            let mut rng = rand::thread_rng();
            (0..clean_frames.len()).map(|_| {
                let index = (rng.gen::<f64>() * amount * 2.0 - amount).round() as i32;
                let safe_index = (index as i32).rem_euclid(clean_frames.len() as i32) as usize;
                clean_frames[safe_index].clone()
            }).collect()
        },
        "overlap" => {
            let pulse_len = opt.countframes;
            let pulse_ryt = opt.positframes;
            clean_frames.chunks(pulse_ryt)
                .flat_map(|chunk| chunk.iter().take(pulse_len).cloned())
                .collect()
        },
        _ => {
            eprintln!("Mode not implemented, using void");
            clean_frames.to_vec()
        }
    }
}

pub fn assemble_output_file(fileout: &PathBuf, temp_hdrl: &PathBuf, temp_movi: &PathBuf, temp_idx1: &PathBuf, final_frames: &[Frame]) -> io::Result<()> {
    let mut output = File::create(fileout)?;

    io::copy(&mut File::open(temp_hdrl)?, &mut output)?;

    output.write_all(b"movi")?;
    let movi_file = File::open(temp_movi)?;
    let mmap = unsafe { Mmap::map(&movi_file)? };

    for frame in final_frames {
        output.write_all(&mmap[frame.offset..frame.offset + frame.size])?;
    }

    io::copy(&mut File::open(temp_idx1)?, &mut output)?;

    Ok(())
}

pub fn process_video(opt: &Opt) -> io::Result<PathBuf> {
    let timer = std::time::Instant::now();
    let temp_dir = tempfile::tempdir()?;
    let temp_hdrl = temp_dir.path().join("hdrl.bin");
    let temp_movi = temp_dir.path().join("movi.bin");
    let temp_idx1 = temp_dir.path().join("idx1.bin");

    let movi_marker_pos = bstream_until_marker(&opt.input, &temp_hdrl, Some(b"movi"), 0)?;
    let idx1_marker_pos = bstream_until_marker(&opt.input, &temp_movi, Some(b"idx1"), movi_marker_pos)?;
    bstream_until_marker(&opt.input, &temp_idx1, None, idx1_marker_pos)?;

    let frame_table = build_frame_table(&temp_movi, opt.audio)?;

    let mut clean_frames = Vec::new();
    let max_frame_size = frame_table.iter().map(|f| f.size).max().unwrap_or(0);

    if opt.firstframe {
        if let Some(first_video_frame) = frame_table.iter().find(|f| f.frame_type == FrameType::Video) {
            clean_frames.push(first_video_frame.clone());
        }
    }

    for frame in &frame_table {
        if frame.size as f32 <= (max_frame_size as f32 * opt.kill) {
            clean_frames.push(frame.clone());
        }
    }

    let final_frames = process_frames(&clean_frames, &opt);

    let cname = if opt.countframes > 1 { format!("-c{}", opt.countframes) } else { String::new() };
    let pname = if opt.positframes > 1 { format!("-n{}", opt.positframes) } else { String::new() };
    let fileout = opt.input.with_file_name(format!("{}-{}{}{}.avi", 
        opt.input.file_stem().unwrap().to_str().unwrap(), 
        opt.mode, cname, pname));

    assemble_output_file(&fileout, &temp_hdrl, &temp_movi, &temp_idx1, &final_frames)?;

    println!("> Done! Output file: {:?}", fileout);
    println!("> Total time: {:.2?}", timer.elapsed());

    Ok(fileout)
}