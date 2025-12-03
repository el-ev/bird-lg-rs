use std::process::Stdio;

use tokio::process::Command;

use crate::config::Config;

#[derive(Debug, Clone, Copy)]
pub enum IpVersion {
    V4,
    V6,
    Any,
}

pub fn build_traceroute_command(
    config: &Config,
    target: &str,
    version: IpVersion,
) -> Option<Command> {
    let bin = config.traceroute_bin.as_ref()?;
    let mut cmd = Command::new(bin);

    cmd.arg(target);

    for arg in &config.traceroute_args {
        cmd.arg(arg);
    }

    match version {
        IpVersion::V4 => {
            cmd.arg("-4");
        }
        IpVersion::V6 => {
            cmd.arg("-6");
        }
        IpVersion::Any => {}
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    Some(cmd)
}
