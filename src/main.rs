use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use directories::{BaseDirs, ProjectDirs};
use env_logger::Env;
use log::{debug, warn};
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const CONFIG_DIR_NAME: &str = "starship";
const LOG_VAR: &str = "STARSHIP_PROFILES_LOG";

#[derive(Debug, Parser)]
struct Cli {
    /// Force the use of a specific profile (regardless of CWD)
    #[clap(short, long)]
    profile: Option<String>,

    /// Print starship's help message (instead of this program's help message)
    #[clap(long)]
    starship_help: bool,

    /// Rest of args passed to starship
    rest: Vec<String>,
}

fn expand_home(s: &str, home: &str) -> String {
    s.replace("~", home)
}

#[derive(Debug, Deserialize)]
struct Profile {
    name: String,
    #[serde(default)]
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
        debug!("No profile matching CWD '{}'", cwd.display());
        None
    }
}

fn find_starship() -> Result<Option<PathBuf>> {
    let this_program = env::current_exe()?;
    let starship = which::which_all("starship")?.find(|p| *p != this_program);
    Ok(starship)
}

fn get_profile_name(
    arg_profile: Option<String>,
    config: &Config,
    home_dir: &Path,
) -> Result<Option<String>> {
    if let Some(profile) = arg_profile {
        // Always take user specified profile if passed as argument
        return Ok(Some(profile));
    }
    let cwd = env::current_dir()?;
    let profile_name = config.matching_profile(&cwd, home_dir);
    Ok(profile_name)
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().filter(LOG_VAR)).init();

    let Some(starship) = find_starship()? else {
        bail!("Unable to find starship exec");
    };

    let Some(base_dirs) = BaseDirs::new() else {
        bail!("Unable to get XDG base dirs");
    };
    let home_dir = base_dirs.home_dir();

    let Some(proj_dirs) = ProjectDirs::from("", "", CONFIG_DIR_NAME) else {
        bail!("Unable to get XDG proj dirs");
    };

    let config_dir = proj_dirs.config_dir();
    let config_file = config_dir.join("profiles.toml");

    let args = Cli::parse();
    let mut starship_args = args.rest;
    if args.starship_help {
        starship_args.insert(0, String::from("--help"));
    }

    let mut cmd = &mut Command::new(&starship);
    cmd = cmd.args(starship_args);

    match Config::from_file(&config_file)? {
        Some(config) => {
            debug!("{:?}", config);

            if let Some(profile_name) = get_profile_name(args.profile, &config, home_dir)? {
                let profiles_dir = config_dir.join("profiles");

                if !profiles_dir.is_dir() {
                    bail!("Profiles dir is not a valid directory");
                }
                let profile = format!("{profile_name}.toml");
                let profile = profiles_dir.join(profile);
                debug!("Using profile: {}", profile.display());

                let starship_config = profile.display().to_string();
                cmd = cmd.env("STARSHIP_CONFIG", starship_config);
            }
        }
        None => warn!("No profiles config found"),
    };

    debug!("Running command {:#?}", cmd);
    let e = cmd.exec();
    bail!("Error running subcommand: {:?}", e)
}
