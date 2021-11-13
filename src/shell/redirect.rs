use nix::{
    fcntl::{open, OFlag},
    sys::stat::Mode,
    unistd::{close, dup, dup2},
};
use std::{io, os::unix::io::RawFd};

pub fn redirect_stdin(infile: Option<&str>) -> io::Result<Option<RawFd>> {
    if let Some(infile) = infile {
        debug!("Redirecting stdin to {}", infile);
        let fd = open(infile, OFlag::O_RDONLY, Mode::empty())?;
        let fdold = dup(0).unwrap();
        dup2(fd, 0).unwrap();
        close(fd).unwrap();
        Ok(Some(fdold))
    } else {
        Ok(None)
    }
}

pub fn restore_stdin(fdold: Option<RawFd>) {
    if let Some(fdold) = fdold {
        debug!("Restoring stdin");
        dup2(fdold, 0).unwrap();
        close(fdold).unwrap();
    }
}

pub fn redirect_stdout(outfile: Option<&str>) -> io::Result<Option<RawFd>> {
    if let Some(outfile) = outfile {
        debug!("Redirecting stdout to {}", outfile);
        let flag = OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC;
        let mode = Mode::S_IRUSR | Mode::S_IWUSR;
        let fd = open(outfile, flag, mode)?;
        let fdold = dup(1).unwrap();
        dup2(fd, 1).unwrap();
        close(fd).unwrap();
        Ok(Some(fdold))
    } else {
        Ok(None)
    }
}

pub fn restore_stdout(fdold: Option<RawFd>) {
    if let Some(fdold) = fdold {
        debug!("Restoring stdout");
        dup2(fdold, 1).unwrap();
        close(fdold).unwrap();
    }
}
