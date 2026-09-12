use crate::builder;
use crate::config::Config;
use anyhow::{Context, Result};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

pub fn watch_and_serve(config: &Config) -> Result<()> {
    // Initial build
    builder::build(config)?;

    println!("👀 Watching for changes in `{}`... (Press Ctrl+C to stop)", config.input.display());

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: std::result::Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        NotifyConfig::default(),
    )
    .context("Failed to initialize file watcher")?;

    watcher
        .watch(&config.input, RecursiveMode::Recursive)
        .with_context(|| format!("Failed to watch directory: {}", config.input.display()))?;

    let debounce_duration = Duration::from_millis(300);
    let mut last_rebuild = Instant::now();

    loop {
        match rx.recv() {
            Ok(event) => {
                // Ignore access events
                if event.kind.is_access() {
                    continue;
                }

                if last_rebuild.elapsed() >= debounce_duration {
                    println!("\n🔄 Change detected ({:?}). Rebuilding...", event.paths);
                    if let Err(e) = builder::build(config) {
                        eprintln!("  ❌ Build error: {}", e);
                    }
                    last_rebuild = Instant::now();
                }
            }
            Err(e) => {
                eprintln!("Watcher channel closed: {}", e);
                break;
            }
        }
    }

    Ok(())
}

