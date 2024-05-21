use expanduser::expanduser;
use std::fs;
use std::io::Read;
use std::process::{Command, Stdio};
use toml::Value;

struct Config {
    backups_to_keep: usize,
    snapshots_dir: String,
}

fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut config_path = dirs::home_dir().unwrap();
    config_path.push(".config/btrfs_backup/config");

    let mut backups_to_keep = 5; // Default value
    let mut snapshots_dir = "~/.snapshots".to_string(); // Default value

    if config_path.exists() {
        let mut file = fs::File::open(&config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let value = contents.parse::<Value>()?;

        if let Some(btk) = value.get("backups_to_keep").and_then(Value::as_integer) {
            backups_to_keep = btk as usize;
        }

        if let Some(sdir) = value.get("snapshots_dir").and_then(Value::as_str) {
            snapshots_dir = sdir.to_string();
        }
    }

    Ok(Config {
        backups_to_keep,
        snapshots_dir: expanduser(&snapshots_dir)?
            .into_os_string()
            .into_string()
            .unwrap(),
    })
}
fn run_command_with_sudo(command: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("sudo")
        .arg(command)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(format!(
            "Command {:?} {:?} failed with status {:?}",
            command, args, status
        )
        .into());
    }

    Ok(())
}

fn create_snapshot(source: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    run_command_with_sudo("btrfs", &["subvolume", "snapshot", source, dest])
}

fn remove_snapshot(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    run_command_with_sudo("btrfs", &["subvolume", "delete", path])
}

fn manage_snapshots(dir: &str, backups_to_keep: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(Result::ok).collect();

    entries.sort_by_key(|entry| entry.metadata().unwrap().modified().unwrap());

    while entries.len() > backups_to_keep {
        if let Some(entry) = entries.first() {
            remove_snapshot(entry.path().to_str().unwrap())?;
            entries.remove(0);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = read_config()?;

    let home_snapshot_dir = format!("{}/home", config.snapshots_dir);
    let root_snapshot_dir = format!("{}/root", config.snapshots_dir);

    fs::create_dir_all(&home_snapshot_dir)?;
    fs::create_dir_all(&root_snapshot_dir)?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let home_snapshot_path = format!("{}/home_snapshot_{}", home_snapshot_dir, timestamp);
    let root_snapshot_path = format!("{}/root_snapshot_{}", root_snapshot_dir, timestamp);

    create_snapshot("/home", &home_snapshot_path)?;
    create_snapshot("/", &root_snapshot_path)?;

    manage_snapshots(&home_snapshot_dir, config.backups_to_keep)?;
    manage_snapshots(&root_snapshot_dir, config.backups_to_keep)?;

    println!("Backup completed successfully.");
    Ok(())
}
