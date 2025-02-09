use std::{
    io::BufRead,
    path::{Path, PathBuf},
    process::Stdio,
};

use tokio::{fs::create_dir_all, process::Command};

use crate::{config::DOWNLOADS_DIR_NAME, mpd::Mpd};


pub(super) fn download_dir_name() -> String {
    DOWNLOADS_DIR_NAME.into()
}

fn download_dir(mpd: &Mpd) -> PathBuf {
    Path::new(mpd.library_directory()).join(download_dir_name())
}

pub(super) async fn create_download_dir(mpd: &Mpd) -> anyhow::Result<()> {
    create_dir_all(download_dir(mpd)).await?;
    Ok(())
}

/// TODO: Download and postprocess song outside of library, then move it in
/// so we don't fight with MPD for file access.
/// Use [tempfile](https://docs.rs/tempfile/latest/tempfile/).
pub(super) async fn download_link(mpd: &Mpd, url: &str) -> anyhow::Result<Vec<String>> {
    let mut command = Command::new("yt-dlp");
    command
        .current_dir(download_dir(mpd))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .args(&[
            "--verbose",
            "--prefer-free-formats",
            "--format",
            "bestaudio/best",
            "--extract-audio",
            "--audio-format",
            "flac",
            "--embed-thumbnail",
            "--embed-metadata",
            "--no-force-overwrites",
            "--restrict-filenames",
            "--no-overwrites",
            "--no-post-overwrites",
            "--output",
            "%(uploader)s - %(title)s [%(extractor)s-%(id)s].%(ext)s",
            "--print",
            "after_move:filepath",
        ])
        .arg(url);

    let filenames: Vec<String> = command
        .spawn()?
        .wait_with_output()
        .await?
        .stdout
        .lines()
        .map(|l| {
            Ok(Path::new(&l?)
                .file_name()
                .expect("All paths should have filenames")
                .to_str()
                .expect("All filenames should be valid UTF-8")
                .to_string())
        })
        .collect::<Result<_, std::io::Error>>()?;

    Ok(filenames)
}
