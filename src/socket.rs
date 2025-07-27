use anyhow::Result;
use std::path::PathBuf;

pub fn get_socket_path() -> Result<PathBuf> {
    let uid = unsafe { libc::getuid() };
    let path = PathBuf::from(format!("/tmp/t_trace.{}.sock", uid));
    Ok(path)
}
