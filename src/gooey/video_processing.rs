use crate::gooey::runtime;

use std::path::PathBuf;
use std::process::Command;
use tokio::process::Command as TokioCommand;
use tokio::task;

pub fn ffmpeg_list_codecs() -> Result<(), std::io::Error> {
    Command::new("ffmpeg")
        .args(&["-codecs"])
        .status()?;
    Ok(())
}

pub fn ffmpeg_to_avi(input: &PathBuf, force: bool, &mut ref mut using_existing: &mut bool) -> Result<PathBuf, std::io::Error> {
    // make subdir 🍅/ in same directory as file and output the file there
    let file_name = input.file_name().unwrap_or_default();
    let parent = input.parent().unwrap_or(std::path::Path::new("🍅"));
    let output = parent.join("🍅").join(file_name).with_extension("avi");

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

pub fn ffmpeg_to_mp4(input: &PathBuf, fast: bool) -> Result<PathBuf, std::io::Error> {
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

pub async fn async_ffmpeg_to_mp4(input: PathBuf, fast: bool) -> Result<PathBuf, std::io::Error> {
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

pub fn try_ffplay(path: &PathBuf) -> Result<(), std::io::Error> {
    Command::new("ffplay")
    .arg(path)
        .status()?;
    Ok(())
}
pub async fn async_try_ffplay(path: PathBuf) -> Result<(), std::io::Error> {
    TokioCommand::new("ffplay")
        .arg(path)
        .status()
        .await?;
    Ok(())
}


// spawners for async tasks
// pub pub fn spawn_ffmpeg_to_avi(input: PathBuf, force: bool) -> task::JoinHandle<Result<PathBuf, std::io::Error>> {
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