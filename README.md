# file-retention-policy

CLI tool to delete old files based on a retention policy inspired by proxmox.

Parses the date out of file names based on a defined pattern.

## Config Example

```toml
[retention]
keep-last = 1
keep-hourly = 0
keep-daily = 7
keep-weekly = 4
keep-monthly = 3
keep-yearly = 1

[[paths]]
path = "/var/backups/consul"
file-pattern = "{name}_{year}-{month}-{day}T{hour}:{minutes}{TZ}.bck"

[[paths]]
path = "/var/backups/pg"
file-pattern = "{year}-{month}-{day}T{hour}:{minutes}{TZ}"
```

## Config Explanation

### `retention`

Configure the retention policy for each path.
You can see how your configured retention applies to files
in [this simulator](https://pbs.proxmox.com/docs/prune-simulator/) for the proxmox backup server.

#### keep-last <N>

Keep the last <N> backup snapshots.

#### keep-hourly <N>

Keep backups for the last <N> hours. If there is more than one backup for a single hour, only the latest is kept. Hours
without backups do not count.

#### keep-daily <N>

Keep backups for the last <N> days. If there is more than one backup for a single day, only the latest is kept. Days
without backups do not count.

#### keep-weekly <N>

Keep backups for the last <N> weeks. If there is more than one backup for a single week, only the latest is kept. Weeks
without backups do not count.
Note:

Weeks start on Monday and end on Sunday. The software uses the ISO week date system and handles weeks at the end of the
year correctly.

#### keep-monthly <N>

Keep backups for the last <N> months. If there is more than one backup for a single month, only the latest is kept.
Months without backups do not count.

#### keep-yearly <N>

Keep backups for the last <N> years. If there is more than one backup for a single year, only the latest is kept. Years
without backups do not count.

### `paths`

You can configure as many paths as you want. They will be processed sequentially.

#### path

The path to the directory where the backups are stored.

#### file-pattern

The file pattern will be converted to a regex pattern to extract the date from the file name. Each placeholder can only be supplied once.

The following placeholders are supported:

| Placeholder | Description                               |
|-------------|-------------------------------------------|
| {year}      | year (e.g., 2024)                         |
| {month}     | month (1..12)                             |
| {month_abr} | abbreviated month name (e.g., Jan)        |
| {day}       | day (1..31)                               |
| {hour}      | hour (0..23)                              |
| {minutes}   | minutes (0..59)                           |
| {seconds}   | seconds (0..59)                           |
| {TZ}        | timezone (e.g., +02:00)                   |
| {name}      | dynamic match with at least one character |
