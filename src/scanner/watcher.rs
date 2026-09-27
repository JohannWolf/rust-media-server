use notify::{
    Config as NotifyConfig,
    Event,
    RecommendedWatcher,
    RecursiveMode,
    Result as NotifyResult,
    Watcher,
};
use std::path::Path;
use tokio::sync::mpsc;

/// Starts watching a directory for filesystem changes.
///
/// The watcher runs independently from the HTTP server and sends
/// filesystem events through a Tokio channel.
pub async fn watch_directory(
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a Tokio channel.
    //
    // The watcher will send events through `tx`.
    // Another task will receive them through `rx`.
    let (tx, mut rx) = mpsc::channel::<NotifyResult<Event>>(100);

    // `notify` uses a callback when an operating-system filesystem
    // event occurs.
    //
    // The callback itself is synchronous, so we use `blocking_send`
    // to send the event into the Tokio channel.
    let mut watcher = RecommendedWatcher::new(
        move |event| {
            // We don't want to panic if the channel is closed.
            // If the application is shutting down, this simply fails.
            let _ = tx.blocking_send(event);
        },
        NotifyConfig::default(),
    )?;

    // Watch the directory recursively.
    watcher.watch(path, RecursiveMode::Recursive)?;

    println!("Watching media directory: {}", path.display());

    // Keep processing events until the application stops.
    while let Some(result) = rx.recv().await {
        match result {
            Ok(event) => {
                println!("Filesystem event: {:?}", event);
            }

            Err(error) => {
                eprintln!("Filesystem watcher error: {}", error);
            }
        }
    }

    Ok(())
}