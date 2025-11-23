use crate::state::Protocol;
use regex::Regex;

pub fn parse_protocols(output: &str) -> Vec<Protocol> {
    let mut protocols = Vec::new();
    // Regex to match: Name Proto Table State Since Info
    // Example: "bgp1 BGP master4 up 10:00:00 Established"
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
