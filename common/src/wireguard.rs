use crate::{
    humanize::{humanize_bytes, humanize_duration},
    models::WireGuardPeer,
};

pub fn parse_wireguard_dump(output: &str) -> Vec<WireGuardPeer> {
    let mut peers = Vec::new();

    for line in output.lines() {
        let fields: Vec<&str> = line.split('\t').collect();

        // let public_key = fields[1].to_string();
        // let endpoint = if fields[3] != "(none)" {
        //     Some(fields[3].to_string())
        // } else {
        //     None
        // };

        let name = fields[0].to_string();
        let latest_handshake_ts: i64 = fields[5].parse().unwrap_or(0);
        let latest_handshake = humanize_duration(latest_handshake_ts);

        let rx_bytes: u64 = fields[6].parse().unwrap_or(0);
        let tx_bytes: u64 = fields[7].parse().unwrap_or(0);

        peers.push(WireGuardPeer {
            name,
            // public_key,
            // endpoint,
            latest_handshake,
            transfer_rx: humanize_bytes(rx_bytes),
            transfer_tx: humanize_bytes(tx_bytes),
        });
    }

    peers.sort_by(|a, b| a.name.cmp(&b.name));

    peers
}

#[cfg(test)]
mod tests {
    use super::parse_wireguard_dump;

    #[test]
    fn parses_and_sorts_wireguard_peers() {
        let output = "\
peer-b\tpub\t(redacted)\t198.51.100.1:51820\tallowed\t0\t1024\t2048\tkeepalive\n\
peer-a\tpub\t(redacted)\t198.51.100.2:51820\tallowed\t3600\t10\t20\tkeepalive\n";

        let peers = parse_wireguard_dump(output);

        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].name, "peer-a");
        assert_eq!(peers[1].name, "peer-b");
        assert_eq!(peers[0].transfer_rx, "10 B");
    }
}
