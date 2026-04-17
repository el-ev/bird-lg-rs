use std::{os::unix::process::CommandExt, process::Command};

fn main() {
    unsafe {
        if libc::setuid(0) != 0 {
            eprintln!("setuid: {}", std::io::Error::last_os_error());
            std::process::exit(1);
        }
        if libc::setgid(0) != 0 {
            eprintln!("setgid: {}", std::io::Error::last_os_error());
            std::process::exit(1);
        }
    }

    let err = Command::new("/usr/bin/wg")
        .arg("show")
        .arg("all")
        .arg("dump")
        .exec();

    eprintln!("execv: {}", err);
    std::process::exit(1);
}
