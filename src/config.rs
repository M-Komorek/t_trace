use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct AppConfig {
    pub stats_path: PathBuf,
    pub temp_path: PathBuf,
}

impl AppConfig {
    pub fn new() -> io::Result<Self> {
        let data_dir = get_data_dir()?;
        Ok(AppConfig {
            stats_path: data_dir.join("stats.json"),
            temp_path: data_dir.join(".temp_entry"),
        })
    }
}

fn get_data_dir() -> io::Result<PathBuf> {
    let home_dir_str = env::var("HOME").map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME environment variable not set.",
        )
    })?;
    let data_dir = PathBuf::from(home_dir_str).join(".t-trace");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }
    Ok(data_dir)
}
