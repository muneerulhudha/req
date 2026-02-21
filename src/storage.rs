use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};

use crate::model::AppState;

pub fn state_path() -> Result<PathBuf> {
    let config = dirs::config_dir().ok_or_else(|| anyhow!("No config directory available"))?;
    Ok(config.join("req").join("state.json"))
}

pub fn load_state() -> Result<AppState> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(AppState::default());
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_state(state: &AppState) -> Result<()> {
    let path = state_path()?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}
