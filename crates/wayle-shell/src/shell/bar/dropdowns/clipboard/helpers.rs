use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::warn;

use super::messages::ClipboardEntry;

const MAX_PREVIEW_CHARS: usize = 60;

pub async fn load_entries() -> Vec<ClipboardEntry> {
    let output = match Command::new("cliphist").arg("list").output().await {
        Ok(o) => o,
        Err(e) => {
            warn!(error = %e, "clipboard: failed to run cliphist list");
            return vec![];
        }
    };

    if !output.status.success() {
        warn!(exit_code = ?output.status.code(), "clipboard: cliphist list returned error");
        return vec![];
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(parse_entry)
        .collect()
}

fn parse_entry(line: &str) -> Option<ClipboardEntry> {
    let tab_pos = line.find('\t')?;
    let content = &line[tab_pos + 1..];

    if content.is_empty() {
        return None;
    }

    let preview = if content.chars().count() > MAX_PREVIEW_CHARS {
        let truncated: String = content.chars().take(MAX_PREVIEW_CHARS).collect();
        format!("{truncated}…")
    } else {
        content.to_owned()
    };

    Some(ClipboardEntry {
        raw: line.to_owned(),
        preview,
    })
}

pub async fn copy_entry(raw: &str) {
    let mut decode = match Command::new("cliphist")
        .arg("decode")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "clipboard: failed to spawn cliphist decode");
            return;
        }
    };

    if let Some(mut stdin) = decode.stdin.take() {
        let _ = stdin.write_all(raw.as_bytes()).await;
    }

    let decoded = match decode.wait_with_output().await {
        Ok(o) if o.status.success() => o.stdout,
        Ok(o) => {
            warn!(exit_code = ?o.status.code(), "clipboard: cliphist decode failed");
            return;
        }
        Err(e) => {
            warn!(error = %e, "clipboard: cliphist decode wait failed");
            return;
        }
    };

    let mut copy = match Command::new("wl-copy")
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "clipboard: failed to spawn wl-copy");
            return;
        }
    };

    if let Some(mut stdin) = copy.stdin.take() {
        let _ = stdin.write_all(&decoded).await;
    }

    let _ = copy.wait().await;
}
