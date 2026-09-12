use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tracing_subscriber::fmt::MakeWriter;

use crate::protocol::CommandSender;

/// Install a tracing subscriber that mirrors logs to `logMessage` and a file.
pub fn init_tracing(sender: CommandSender, plugin_uuid: &str) {
    let writer = StreamDeckWriter {
        sender,
        file: Arc::new(Mutex::new(open_log_file(plugin_uuid))),
    };
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(writer)
        .with_ansi(false)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
}

#[derive(Clone)]
struct StreamDeckWriter {
    sender: CommandSender,
    file: Arc<Mutex<Option<File>>>,
}

impl Write for StreamDeckWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let _ = self.sender.log_message(trimmed);
            }
        }
        if let Ok(mut file) = self.file.lock() {
            if let Some(file) = file.as_mut() {
                file.write_all(buf)?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Ok(mut file) = self.file.lock() {
            if let Some(file) = file.as_mut() {
                file.flush()?;
            }
        }
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for StreamDeckWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

fn open_log_file(plugin_uuid: &str) -> Option<File> {
    let path = log_path(plugin_uuid)?;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    OpenOptions::new().create(true).append(true).open(path).ok()
}

fn log_path(plugin_uuid: &str) -> Option<PathBuf> {
    let file = format!("{plugin_uuid}.log");
    if cfg!(windows) {
        let appdata = std::env::var_os("APPDATA")?;
        Some(
            PathBuf::from(appdata)
                .join("Elgato")
                .join("StreamDeck")
                .join("logs")
                .join(file),
        )
    } else if cfg!(target_os = "macos") {
        let home = std::env::var_os("HOME")?;
        Some(
            PathBuf::from(home)
                .join("Library")
                .join("Logs")
                .join("ElgatoStreamDeck")
                .join(file),
        )
    } else {
        Some(std::env::temp_dir().join(file))
    }
}
