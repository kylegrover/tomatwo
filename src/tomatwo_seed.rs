// tomatwo_lib.rs
use std::process::{Command, Stdio};
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
    pub multiply: i32,
    pub kill_rel: f32,
    pub preview: bool,
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub offset: usize,
    pub size: usize,
    pub rel_size: f32,
    pub frame_type: FrameType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrameType {
    Video,
    Audio,
    Void,
}

pub fn process_frames(clean_frames: &[Frame], opt: &Opt) -> (Vec<Frame>, Vec<usize>) {
    let processed_frames = match opt.mode.as_str() {
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
    };
    let processed_sizes = processed_frames.iter().map(|f| f.size).collect();
    (processed_frames, processed_sizes)
}

pub fn simulate_processing(mut frame_data: Vec<Frame>, steps: &[Opt]) -> Vec<Frame> {
    let orig_frame_count = frame_data.len();
    for step in steps {
        let mut clean_frames = Vec::new();
        let max_frame_size = frame_data.iter().map(|f| f.size).max().unwrap_or(0);
        let mut prev_frame_size = 0;

        // keep first video frame or not
        if step.firstframe {
            if let Some(first_video_frame) = frame_data.iter().find(|f| f.frame_type == FrameType::Video) {
                clean_frames.push(first_video_frame.clone());
                prev_frame_size = first_video_frame.size;
            }
        }

        // clean the list by killing "big" frames and frames with large relative size increases
        for frame in &frame_data {
            let keep_frame = frame.size as f32 <= (max_frame_size as f32 * step.kill) &&
                            (frame.size as f32 <= prev_frame_size as f32 * (1.0 + step.kill_rel));

            if keep_frame {
                clean_frames.push(frame.clone());
            }
            prev_frame_size = frame.size;
        }


        let (processed_frames, _) = process_frames(&clean_frames, &step);
        let mut final_frames = processed_frames;

        if step.multiply > 1 {
            let mut new_frames = Vec::new();
            for frame in final_frames {
                for _ in 0..step.multiply {
                    new_frames.push(frame.clone());
                }
            }
            final_frames = new_frames;
        }
        frame_data = final_frames;
    }
    println!("> Simulated processing: {} -> {} frames using {} steps", 
        orig_frame_count, frame_data.len(), steps.len());
    
    frame_data
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

    let frame_table: Vec<Frame> = mmap.par_windows(4)
        .enumerate()
        .filter_map(|(i, window)| {
            match window {
                b"00dc" => Some(Frame { offset: i, size: 0, rel_size: 0.0, frame_type: FrameType::Video }),
                b"01wb" if include_audio => Some(Frame { offset: i, size: 0, rel_size: 0.0, frame_type: FrameType::Audio }),
                _ => None,
            }
        })
        .collect();

    let mut frame_table: Vec<Frame> = frame_table;
    for i in 0..frame_table.len() {
        frame_table[i].size = if i + 1 < frame_table.len() {
            frame_table[i + 1].offset - frame_table[i].offset
        } else {
            mmap.len() - frame_table[i].offset
        };
    }

    Ok(frame_table)
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
    let mut prev_frame_size = 0;

    // keep first video frame or not
    if opt.firstframe {
        if let Some(first_video_frame) = frame_table.iter().find(|f| f.frame_type == FrameType::Video) {
            clean_frames.push(first_video_frame.clone());
            prev_frame_size = first_video_frame.size;
        }
    }

    // clean the list by killing "big" frames and frames with large relative size increases
    for frame in &frame_table {
        let keep_frame = frame.size as f32 <= (max_frame_size as f32 * opt.kill) &&
                         (frame.size as f32 <= prev_frame_size as f32 * (1.0 + opt.kill_rel));

        if keep_frame {
            clean_frames.push(frame.clone());
        }
        prev_frame_size = frame.size;
    }


    let (processed_frames, _) = process_frames(&clean_frames, &opt);
    let mut final_frames = processed_frames;

    if opt.multiply > 1 {
        let mut new_frames = Vec::new();
        for frame in final_frames {
            for _ in 0..opt.multiply {
                new_frames.push(frame.clone());
            }
        }
        final_frames = new_frames;
    }

    if opt.preview {
        preview_output(&temp_hdrl, &temp_movi, &temp_idx1, &final_frames)?;
        Ok(PathBuf::new())
    } else {
        println!("> Processing complete, writing output file... {:.2?}", timer.elapsed());
    
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
}

pub fn preview_output(temp_hdrl: &PathBuf, temp_movi: &PathBuf, temp_idx1: &PathBuf, final_frames: &[Frame]) -> io::Result<()> {
    let mut ffplay = Command::new("ffplay")
        .args(&["-f", "avi", "-i", "-"])  // Read from stdin
        .stdin(Stdio::piped())
        .spawn()?;

    let mut ffplay_stdin = ffplay.stdin.take().expect("Failed to open ffplay stdin");

    // Write HDRL
    io::copy(&mut File::open(temp_hdrl)?, &mut ffplay_stdin)?;

    // Write MOVI header
    ffplay_stdin.write_all(b"movi")?;

    // Write frames
    let movi_file = File::open(temp_movi)?;
    let mmap = unsafe { Mmap::map(&movi_file)? };

    for frame in final_frames {
        ffplay_stdin.write_all(&mmap[frame.offset..frame.offset + frame.size])?;
    }

    // Write IDX1
    io::copy(&mut File::open(temp_idx1)?, &mut ffplay_stdin)?;

    // Close stdin to signal end of input
    drop(ffplay_stdin);

    // Wait for ffplay to finish
    ffplay.wait()?;

    Ok(())
}

pub fn extract_frame_data(avi_path: &PathBuf) -> io::Result<(Vec<Frame>, usize)> {
    let file = File::open(avi_path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let movi_marker = b"movi";
    let idx1_marker = b"idx1";

    if let (Some(movi_start), Some(idx1_start)) = (
        mmap.windows(movi_marker.len()).position(|window| window == movi_marker),
        mmap.windows(idx1_marker.len()).position(|window| window == idx1_marker)
    ) {
        let movi_data = &mmap[movi_start..idx1_start];

        let frame_table: Vec<Frame> = movi_data.par_windows(4)
            .enumerate()
            .filter_map(|(i, window)| {
                if window == b"00dc" {
                    Some(Frame { offset: movi_start + i, size: 0, rel_size: 0.0, frame_type: FrameType::Video })
                } else {
                    None
                }
            })
            .collect();

        let mut last_frame_size = 0;
        let mut frame_table = frame_table;
        for i in 0..frame_table.len() {
            if i + 1 < frame_table.len() {
                frame_table[i].size = frame_table[i + 1].offset - frame_table[i].offset - 8; // Subtract 8 for chunk header
                frame_table[i].rel_size = frame_table[i].size as f32 / last_frame_size as f32;
                last_frame_size = frame_table[i].size;
            } else {
                frame_table[i].size = idx1_start - frame_table[i].offset - 8; // Use idx1_start as the end of movi data
            };
        }

        let max_frame_size = frame_table.iter().map(|f| f.size).max().unwrap_or(0);

        Ok((frame_table, max_frame_size))
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidData, "Could not find 'movi' or 'idx1' marker in AVI file"))
    }
}