use std::io::Write;

/// Copy text to the clipboard using OSC 52.
/// Works in any compliant terminal, including over SSH and inside tmux
/// (requires `set-clipboard on` in tmux.conf).
pub fn copy(text: &str) {
    let encoded = base64(text.as_bytes());
    let seq = format!("\x1b]52;c;{encoded}\x07");
    let _ = std::io::stdout().write_all(seq.as_bytes());
    let _ = std::io::stdout().flush();
}

fn base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let n = chunk.len();
        let b0 = chunk[0];
        let b1 = if n > 1 { chunk[1] } else { 0 };
        let b2 = if n > 2 { chunk[2] } else { 0 };
        out.push(CHARS[(b0 >> 2) as usize] as char);
        out.push(CHARS[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
        if n > 1 { out.push(CHARS[(((b1 & 0xf) << 2) | (b2 >> 6)) as usize] as char); } else { out.push('='); }
        if n > 2 { out.push(CHARS[(b2 & 0x3f) as usize] as char); } else { out.push('='); }
    }
    out
}
