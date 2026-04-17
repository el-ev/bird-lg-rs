use crate::models::Protocol;

pub fn parse_protocols(output: &str) -> Vec<Protocol> {
    let mut protocols = Vec::new();

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

        let mut parts = line.split_ascii_whitespace();
        if let (Some(name), Some(proto), Some(table), Some(state), Some(since)) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            protocols.push(Protocol {
                name: name.to_string(),
                proto: proto.to_string(),
                table: table.to_string(),
                state: state.to_string(),
                since: since.to_string(),
                info: parts.collect::<Vec<_>>().join(" "),
            });
        }
    }

    protocols
}

pub fn filter_protocol_details(raw: &str) -> String {
    const PROTOCOL_HEADERS: [&str; 6] = ["Name", "Proto", "Table", "State", "Since", "Info"];

    raw.lines()
        .filter(|line| !PROTOCOL_HEADERS.iter().all(|header| line.contains(header)))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{filter_protocol_details, parse_protocols};

    #[test]
    fn parses_protocol_rows_and_preserves_multi_word_info() {
        let output = "\
Name Proto Table State Since Info\n\
peer1 BGP master4 up 2026-04-17 Established session ok\n\
peer2 BGP master6 start 2026-04-17 Connect retry\n";

        let parsed = parse_protocols(output);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "peer1");
        assert_eq!(parsed[0].info, "Established session ok");
        assert_eq!(parsed[1].info, "Connect retry");
    }

    #[test]
    fn removes_protocol_table_header_from_details() {
        let filtered = filter_protocol_details(
            "Name Proto Table State Since Info\npeer1 BGP master4 up 2026-04-17 Established",
        );

        assert_eq!(filtered, "peer1 BGP master4 up 2026-04-17 Established");
    }
}
