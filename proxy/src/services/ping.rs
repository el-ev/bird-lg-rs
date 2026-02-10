use std::process::Stdio;

use tokio::process::Command;

use crate::{config::Config, services::traceroute::IpVersion};

pub fn build_ping_command(config: &Config, target: &str, version: IpVersion) -> Option<Command> {
    let bin = config.ping_bin.as_ref()?;
    let mut cmd = Command::new(bin);

    // Limit to 5 packets
    cmd.arg("-c").arg("5");

    match version {
        IpVersion::V4 => {
            cmd.arg("-4");
        }
        IpVersion::V6 => {
            cmd.arg("-6");
        }
        IpVersion::Any => {}
    }

    cmd.arg(target);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    Some(cmd)
}
