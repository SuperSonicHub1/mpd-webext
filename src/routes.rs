use axum::extract::Query;

use axum::response::Redirect;

use axum::response::IntoResponse;
use maud::{html, DOCTYPE};
use serde::Deserialize;

use crate::config::NAME;
use crate::job_manager::Job;
use crate::job_manager::JobManager;
use crate::mpd::Mpd;
use crate::server_util::AppResult;

use std::sync::Arc;

use axum::Extension;

use axum_messages::Messages;


pub(crate) async fn index(messages: Messages, manager: Extension<Arc<JobManager>>, mpd: Extension<Arc<Mpd>>) -> AppResult<impl IntoResponse> {
    let playing = mpd.currently_playing().await?;
    let maybe_catalog = mpd.get_downloads_playlist().await.ok();
    let jobs = manager.jobs();
    Ok(html! {
        (DOCTYPE)
        head {
            title { (NAME) }
        }
        @if !messages.is_empty() {
            details open {
                summary {
                    h5 { "Messages" }
                }
                ul {
                    @for message in messages {
                        li { (message) }
                    }
                }
            }
        }
        h1 { (NAME) }
        p {
            "Currently playing: 「"
            (playing)
            "」"
        }
        p {
            form action="/toggle-playback" method="get" {
                input type="submit" value="Toggle playback";
            }
        }
        p {
            @if manager.ongoing() {
                "Job ongoing, "
            }
            (jobs) " job"
            @if jobs == 1 {
                ""
            } @else {
                "s"
            }
            " in queue"
        }
        p {
            form action="/download" method="get" {
                label {
                    "Download link, add to queue: "
                    input type="input" id="url" name="url" placeholder="https://youtube.com/watch?v=dQw4w9WgXcQ";
                }
                input type="submit" value="Download";
            }
        }
        details open {
            summary { 
                h2 { "Catalog" }
            }
            @match maybe_catalog {
                Some(catalog) => {
                    table {
                        thead {
                            th scope="col" { "Title" }
                            th scope="col" { "Artists" }
                            th scope="col" { "Queue" }
                        }
                        tbody {
                            @for song in catalog {
                                @let title = song
                                    .title()
                                    .or(song.file_path().to_str())
                                    .unwrap_or("[no name]");
                                tr {
                                    th scope="row" { (title) }
                                    td { (song.artists().join("& ")) }
                                    td { 
                                        form action="/queue" method="get" {
                                            input type="hidden" name="uri" value=(song.url);
                                            input type="submit" value="Queue";
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                None => p { "No songs; add one!" }
            }
            
        }
    })
}

pub(crate) async fn toggle_playback(
    messages: Messages,
    mpd: Extension<Arc<Mpd>>,
) -> AppResult<impl IntoResponse> {
    mpd.toggle().await?;
    messages.info("Playback toggled.");
    Ok(Redirect::to("/"))
}

#[derive(Deserialize)]
pub(crate) struct Download {
    pub(crate) url: String,
}

pub(crate) async fn download(
    messages: Messages,
    manager: Extension<Arc<JobManager>>,
    query: Query<Download>,
) -> AppResult<impl IntoResponse> {
    manager.push(Job::Download { url: query.url.clone() });
    messages.info("Link added to job queue.");
    Ok(Redirect::to("/"))
}

#[derive(Deserialize)]
pub(crate) struct Queue {
    pub(crate) uri: String,
}

pub(crate) async fn queue(
    messages: Messages,
    mpd: Extension<Arc<Mpd>>,
    query: Query<Queue>,
) -> AppResult<impl IntoResponse> {
    mpd.queue(&query.uri).await?;
    messages.info("Song added to queue.");
    Ok(Redirect::to("/"))
}
