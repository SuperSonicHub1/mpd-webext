///! Poor person's config file for mpd-webext

pub(crate) static SERVER_ADDR: &'static str = "0.0.0.0:3000";
pub(crate) static MPD_SERVER_ADDR: &'static str = "localhost:6600";
pub(crate) static MUSIC_DIRECTORY: &'static str = r#"C:/Users/kylea/Music/Listening"#;
pub(crate) static NAME: &'static str = "mpd-webext";
pub(crate) static DOWNLOADS_PLAYLIST_NAME: &'static str = "mpd-webext Downloads";
pub(crate) static DOWNLOADS_DIR_NAME: &'static str = DOWNLOADS_PLAYLIST_NAME;
pub(crate) static RESCAN_TIME: u64 = 5;
