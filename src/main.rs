use anyhow::bail;
use anyhow::Result;
use directories::{BaseDirs, ProjectDirs};
use env_logger::Env;
use log::{debug, info, warn};
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const CONFIG_DIR_NAME: &str = "starship";
const DEFAULT_STARSHIP_CONFIG: &str = "~/.config/starship.toml";
const LOG_VAR: &str = "STARSHIP_PROFILES_LOG";

fn expand_home(s: &str, home: &str) -> String {
    s.replace("~", home)
}

#[derive(Debug, Deserialize)]
struct Profile {
    name: String,
    patterns: Vec<String>,
}

impl Profile {
    pub fn matches(&self, cwd: &str, home: &str) -> bool {
        for pattern in &self.patterns {
            let re = Regex::new(&expand_home(pattern, home)).unwrap();
            if re.captures(cwd).is_some() {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(rename = "profile")]
    profiles: Vec<Profile>,
}

impl Config {
    pub fn from_file(config_file: &Path) -> Result<Option<Self>> {
        if !config_file.is_file() {
            return Ok(None);
        }
        let contents = fs::read_to_string(config_file)?;
        let config = toml::from_str(&contents)?;
        Ok(Some(config))
    }

    pub fn matching_profile(&self, cwd: &Path, home: &Path) -> Option<String> {
        let home = home.to_str().unwrap();
        let cwd_str = expand_home(cwd.to_str().unwrap(), home);
        for profile in &self.profiles {
            if profile.matches(&cwd_str, home) {
                return Some(profile.name.clone());
            }
        }
        None
    }
}

fn find_starship() -> Result<Option<PathBuf>> {
    let this_program = env::current_exe()?;
    let starship = which::which_all("starship")?.find(|p| *p != this_program);
    Ok(starship)
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().filter(LOG_VAR)).init();

    let Some(starship) = find_starship()? else {
        bail!("Unable to find starship exec");
    };

    let mut starship_config = DEFAULT_STARSHIP_CONFIG.to_string();

    let Some(base_dirs) = BaseDirs::new() else {
        bail!("Unable to get XDG base dirs");
    };
    let home_dir = base_dirs.home_dir();

    let Some(proj_dirs) = ProjectDirs::from("", "", CONFIG_DIR_NAME) else {
        bail!("Unable to get XDG proj dirs");
    };

    let config_dir = proj_dirs.config_dir();
    let config_file = config_dir.join("profiles.toml");

    match Config::from_file(&config_file)? {
        Some(config) => {
            debug!("{:?}", config);
            let cwd = env::current_dir()?;
            match config.matching_profile(&cwd, &home_dir) {
                Some(profile_name) => {
                    let profiles_dir = config_dir.join("profiles");
                    if !profiles_dir.is_dir() {
                        bail!("Profiles dir is not a valid directory");
                    }

                    let profile = format!("{profile_name}.toml");
                    let profile = profiles_dir.join(profile);

                    debug!("Using profile: {}", profile.display());

                    starship_config = profile.display().to_string();
                }
                None => debug!("No matching profile for CWD"),
            }
        }
        None => warn!("No profiles config found"),
    };

    let e = Command::new(&starship)
        // Passthrough args
        // Skip to move off arg0 (program name)
        .args(env::args().skip(1))
        .env("STARSHIP_CONFIG", starship_config)
        .exec();

    bail!("Error running subcommand: {:?}", e)
}
