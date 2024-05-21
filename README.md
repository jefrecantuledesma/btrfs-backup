# btrfs-backup
A script to automate backing up the root and home btrfs subvolumes. Why create a program that has already been created countless other times? Well, it's far more fun to do it yourself.

## Configuration
The configuration file must be created in the following directory: `~/.config/btrfs_backup/config`. Without the config file, the script will use default values and locations. 

Within the config file, there are several options:
- `backups_to_keep = {int}`
- `snapshot_dir = {"string"}`

### Example Configuration
```
backups_to_keep = 5
snapshots_dir = "~/.snapshots"
```
