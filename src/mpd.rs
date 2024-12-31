use std::time::Duration;

use mpd_client::{
    commands::{Add, AddToPlaylist, CurrentSong, GetPlaylist, Rescan, SetPause, Status},
    responses::{PlayState, Song},
    Client,
};
use tokio::{net::TcpStream, time::sleep};

use crate::{config::{DOWNLOADS_PLAYLIST_NAME, MPD_SERVER_ADDR, MUSIC_DIRECTORY}, yt_dlp};



#[derive(Debug)]
pub(super) struct Mpd {
    client: Client,
}

impl Mpd {
    pub(super) async fn new() -> anyhow::Result<Self> {
        let stream: TcpStream = TcpStream::connect(MPD_SERVER_ADDR).await?;
        let (client, _) = Client::connect(stream).await?;
        anyhow::Result::Ok(Self { client })
    }

    pub(super) async fn toggle(&self) -> anyhow::Result<()> {
        let status = self.client.command(Status).await?;
        match status.state {
            PlayState::Stopped => {
                println!("Nothing playing.");
            }
            state => {
                let playing = state == PlayState::Playing;
                self.client.command(SetPause(playing)).await?;
                println!("Play state toggled!");
            }
        };
        Ok(())
    }

    pub(super) async fn currently_playing(&self) -> anyhow::Result<String> {
        let possible_song = self.client.command(CurrentSong).await?;
        let name = if let Some(song) = possible_song {
            song.song.title().unwrap_or("[no name]").into()
        } else {
            "[none]".into()
        };
        Ok(name)
    }

    pub(super) async fn get_downloads_playlist(&self) -> anyhow::Result<Vec<Song>> {
        let songs = self.client.command(GetPlaylist(DOWNLOADS_PLAYLIST_NAME)).await?;
        Ok(songs)
    }

    /// TODO: Make configurable
    pub(super) fn library_directory(&self) -> &str {
        MUSIC_DIRECTORY
    }

    pub(super) async fn download_link(&self, url: &str) -> anyhow::Result<()> {
        let previous_playlist = self.get_downloads_playlist().await?;
        let already_in_playlist = |uri: &str| {
            previous_playlist.iter().any(|song| song.url.contains(uri))
        };

        // Download song
        let files = yt_dlp::download_link(&self, url).await?;

        // Update downloads folder
        self.client.command(Rescan::new()).await?;
        // It seems that the rescan happens asynchronously; a bother.
        sleep(Duration::from_secs(2)).await;

        // Add files to queue and playlist
        for file in files {
            let uri = vec![yt_dlp::download_dir_name(), file].join("/");

            self.client.command(Add::uri(uri.as_str())).await?;

            // Don't add song to playlist if already there
            if !already_in_playlist(uri.as_str()) {
                self.client
                    .command(AddToPlaylist::new(DOWNLOADS_PLAYLIST_NAME, uri.as_str()))
                    .await?;
            }
        }

        Ok(())
    }

    pub(super) async fn queue(&self, uri: &str) -> anyhow::Result<()> {
        self.client.command(Add::uri(uri)).await?;
        Ok(())
    }
}
