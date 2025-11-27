use common::models::Protocol;
use futures_util::{Stream, StreamExt, stream};
use regex::Regex;

pub fn parse_protocols(output: &str) -> Vec<Protocol> {
    let mut protocols = Vec::new();
    let re = Regex::new(r"^(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s*(.*)$").unwrap();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if ["Name", "Proto", "Table", "State", "Since", "Info"]
            .iter()
            .all(|&header| line.contains(header))
        {
            continue;
        }

        if let Some(caps) = re.captures(line) {
            protocols.push(Protocol {
                name: caps[1].to_string(),
                proto: caps[2].to_string(),
                table: caps[3].to_string(),
                state: caps[4].to_string(),
                since: caps[5].to_string(),
                info: caps[6].to_string(),
            });
        }
    }
    protocols
}

pub fn byte_stream_to_lines<S, E>(stream: S) -> impl Stream<Item = Vec<String>>
where
    S: Stream<Item = Result<axum::body::Bytes, E>> + Unpin,
    E: std::fmt::Debug,
{
    stream::unfold((stream, Vec::new()), |(mut stream, mut buf)| async move {
        loop {
            let extract_lines = |buffer: &mut Vec<u8>| -> Vec<String> {
                let mut lines = Vec::new();
                while let Some(i) = buffer.iter().position(|&b| b == b'\n') {
                    let line_bytes: Vec<u8> = buffer.drain(..=i).collect();
                    let mut line = String::from_utf8_lossy(&line_bytes).to_string();
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    if line.ends_with('\r') {
                        line.pop();
                    }
                    lines.push(line);
                }
                lines
            };

            match stream.next().await {
                Some(Ok(bytes)) => {
                    buf.extend_from_slice(&bytes);

                    if buf.contains(&b'\n') {
                        let lines = extract_lines(&mut buf);
                        if !lines.is_empty() {
                            return Some((lines, (stream, buf)));
                        }
                    }
                }
                Some(Err(_)) => {
                    let lines = extract_lines(&mut buf);
                    if !lines.is_empty() {
                        return Some((lines, (stream, buf)));
                    }
                    return None;
                }
                None => {
                    let mut lines = extract_lines(&mut buf);
                    if !buf.is_empty() {
                        let line = String::from_utf8_lossy(&buf).to_string();
                        lines.push(line);
                        buf.clear();
                    }

                    if !lines.is_empty() {
                        return Some((lines, (stream, buf)));
                    }
                    return None;
                }
            }
        }
    })
}
