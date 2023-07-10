use std::{fs, path::PathBuf};

pub fn init() -> anyhow::Result<()> {
    fs::create_dir_all(user_home_path())?;
    fs::create_dir_all(program_root_dir())?;
    fs::create_dir_all(program_projects_dir())?;

    Ok(())
}

pub fn user_home_path() -> PathBuf {
    home::home_dir().expect("Unable to get user home directory")
}

pub fn program_root_dir() -> PathBuf {
    let mut root_dir = user_home_path();
    root_dir.push(".config/teef/");
    root_dir
}

pub fn program_projects_dir() -> PathBuf {
    let mut projects_dir = program_root_dir();
    projects_dir.push("projects/");
    projects_dir
}
