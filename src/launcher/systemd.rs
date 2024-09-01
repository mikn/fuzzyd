use crate::fuzzy::FuzzyItem;
use std::process::Command;
use crate::error::FuzzydError;
use rand::Rng;
use crate::config::SystemdRunConfig;
use std::env;
use regex::Regex;
use std::fs;

pub struct SystemdLauncher {
    dry_run: bool,
    parameters: Vec<String>,
}

impl SystemdLauncher {
    pub fn new(dry_run: bool, config: &SystemdRunConfig) -> Self {
        let parameters = config.parameters.iter()
            .map(|param| Self::expand_env_vars(param))
            .collect();
        SystemdLauncher { 
            dry_run, 
            parameters,
        }
    }

    fn expand_env_vars(command: &str) -> String {
        let re = Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
        re.replace_all(command, |caps: &regex::Captures| {
            env::var(&caps[1]).unwrap_or_else(|_| caps[0].to_string())
        }).to_string()
    }

    pub fn launch(&self, item: &FuzzyItem) -> Result<(), FuzzydError> {
        let command = &item.exec;
        let random_number: u32 = rand::thread_rng().gen_range(100_000..=999_999);
        let unit_cmd_name = format!("app-fuzzyd-{}-{:06}", 
            item.display
                .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != ' ', "")
                .replace("-", "_")
                .replace(" ", "_")
                .to_lowercase(), // Convert to lower case
            random_number);

        let mut cmd = Command::new("/usr/bin/systemd-run");
        for param in &self.parameters {
            cmd.arg(param);
        }
        cmd.arg("--unit")
            .arg(&unit_cmd_name);

        // Split the command into parts manually
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        // Find the executable by iteratively checking from the back
        let mut executable = None;
        for i in (1..=parts.len()).rev() {
            let potential_executable = parts[..i].join(" ");
            if fs::metadata(&potential_executable).is_ok() {
                executable = Some((potential_executable, &parts[i..]));
                break;
            }
        }

        if let Some((executable, args)) = executable {
            cmd.arg(executable);
            for arg in args {
                cmd.arg(arg);
            }
        } else {
            return Err(FuzzydError::LaunchError("Executable not found".to_string()));
        }

        if self.dry_run {
            println!("Dry run: {:?}", cmd);
        } else {
            let mut child = cmd.spawn()
                .map_err(|e| FuzzydError::LaunchError(e.to_string()))?;
            child.wait()
                .map_err(|e| FuzzydError::LaunchError(e.to_string()))?;
        }
        Ok(())
    }
}