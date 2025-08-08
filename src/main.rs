use clap::{Parser, Subcommand};
use libc;
use regex::Regex;
use semver::Version;
use std::env;
use std::ffi::CString;
use std::fs;
use std::net::{TcpStream, Ipv4Addr};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Duration;
use glob::glob;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A modern, descriptive replacement for the 'test' command.",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Commands based on https://linux.die.net/man/1/test

#[derive(Subcommand)]
enum FileCommand {
    /// Checks if a file exists (-e).
    #[clap(name = "exists")]
    Exists { path: String },
    /// Checks if a path is a directory (-d).
    #[clap(name = "directory")]
    Directory { path: String },
    /// Checks if a path is a regular file (-f).
    #[clap(name = "regular")]
    File { path: String },
    /// Checks if a path is a symbolic link (-h, -L). Does not dereference.
    #[clap(name = "symlink")]
    Symlink { path: String },
    /// Checks if a file is a block special file (-b).
    #[clap(name = "block-device")]
    BlockDevice { path: String },
    /// Checks if a file is a character special file (-c).
    #[clap(name = "character-device")]
    CharacterDevice { path: String },
    /// Checks if a file is a named pipe (FIFO) (-p).
    #[clap(name = "named-pipe")]
    NamedPipe { path: String },
    /// Checks if a file is a socket (-S).
    #[clap(name = "socket")]
    Socket { path: String },
    /// Checks if a file exists and has a size greater than zero (-s).
    #[clap(name = "non-empty")]
    NonEmpty { path: String },
    /// Checks if a file is readable by the current (effective) user (-r).
    #[clap(name = "readable")]
    Readable { path: String },
    /// Checks if a file is writable by the current (effective) user (-w).
    #[clap(name = "writable")]
    Writable { path: String },
    /// Checks if a file is executable by the current (effective) user (-x).
    #[clap(name = "executable")]
    Executable { path: String },
    /// Checks if the file has the set-user-ID bit set (-u).
    #[clap(name = "has-suid")]
    Suid { path: String },
    /// Checks if the file has the set-group-ID bit set (-g).
    #[clap(name = "has-sgid")]
    Sgid { path: String },
    /// Checks if the file has the sticky bit set (-k).
    #[clap(name = "has-sticky")]
    Sticky { path: String },
    /// Checks if a file is owned by the effective user ID (-O).
    #[clap(name = "owned-by-effective-user")]
    OwnedByEffectiveUser { path: String },
    /// Checks if a file is owned by the effective group ID (-G).
    #[clap(name = "owned-by-effective-group")]
    OwnedByEffectiveGroup { path: String },
    /// Checks if two files are on the same device and have the same inode number (-ef).
    #[clap(name = "has-same-inode")]
    SameInode { path1: String, path2: String },
    /// Checks if the first file is newer than the second (-nt).
    #[clap(name = "newer-than")]
    Newer { path1: String, path2: String },
    /// Checks if the first file is older than the second (-ot).
    #[clap(name = "older-than")]
    Older { path1: String, path2: String },
    /// Does any file match the given glob pattern
    #[clap(name = "exists-glob")]
    ExistsGlob { pattern: String },
    /// Does any file matching the glob have size > 0
    #[clap(name = "non-empty-glob")]
    NonEmptyGlob { pattern: String },
    /// File size compare (>)
    #[clap(name = "size-gt")]
    FileSizeGt { path: String, bytes: u64 },
    /// File size compare (>=)
    #[clap(name = "size-ge")]
    FileSizeGe { path: String, bytes: u64 },
    /// File size compare (<)
    #[clap(name = "size-lt")]
    FileSizeLt { path: String, bytes: u64 },
    /// File size compare (<=)
    #[clap(name = "size-le")]
    FileSizeLe { path: String, bytes: u64 },
    /// File size compare (=)
    #[clap(name = "size-eq")]
    FileSizeEq { path: String, bytes: u64 },
    /// File mtime older than N seconds
    #[clap(name = "mtime-older-than")]
    FileMtimeOlderThan { path: String, seconds: u64 },
    /// File mtime newer than N seconds
    #[clap(name = "mtime-newer-than")]
    FileMtimeNewerThan { path: String, seconds: u64 },
}

#[derive(Subcommand)]
enum StringCommand {
    /// String equals (=)
    #[clap(name = "equal")]
    Equal { string1: String, string2: String },
    /// String not equals (!=)
    #[clap(name = "not-equals")]
    NotEqual { string1: String, string2: String },
    /// String is empty (-z).
    #[clap(name = "empty")]
    EmptyString { string: String },
    /// String is not empty (-n).
    #[clap(name = "not-empty")]
    NonEmptyString { string: String },
    /// Case-insensitive string equality
    #[clap(name = "equal-ci")]
    EqualCaseInsensitive { string1: String, string2: String },
    /// Regex full or partial match
    #[clap(name = "matches-regex")]
    Regex { string: String, pattern: String },
    /// Case-insensitive regex match
    #[clap(name = "matches-regex-ci")]
    RegexCaseInsensitive { string: String, pattern: String },
    /// String contains substring
    #[clap(name = "contains")]
    Contains { string: String, needle: String },
    /// String contains substring, case-insensitive
    #[clap(name = "contains-ci")]
    ContainsCaseInsensitive { string: String, needle: String },
    /// String starts with prefix
    #[clap(name = "starts-with")]
    StartsWith { string: String, prefix: String },
    /// String starts with prefix, case-insensitive
    #[clap(name = "starts-with-ci")]
    StartsWithCaseInsensitive { string: String, prefix: String },
    /// String ends with suffix
    #[clap(name = "ends-with")]
    EndsWith { string: String, suffix: String },
    /// String ends with suffix, case-insensitive
    #[clap(name = "ends-with-ci")]
    EndsWithCaseInsensitive { string: String, suffix: String },
    /// Is the provided string an integer (base 10)
    #[clap(name = "integer")]
    IsInteger { string: String },
    /// Is the provided string a number (integer or float)
    #[clap(name = "number")]
    IsNumber { string: String },
    /// String is UUID (8-4-4-4-12 hex)
    #[clap(name = "uuid")]
    StringIsUuid { string: String },
    /// String is IPv4 address
    #[clap(name = "ipv4")]
    StringIsIpv4 { string: String },
    /// String is ASCII only
    #[clap(name = "ascii")]
    StringAsciiOnly { string: String },
    /// String length compare (>)
    #[clap(name = "len-gt")]
    StringLenGt { string: String, n: usize },
    /// String length compare (>=)
    #[clap(name = "len-ge")]
    StringLenGe { string: String, n: usize },
    /// String length compare (<)
    #[clap(name = "len-lt")]
    StringLenLt { string: String, n: usize },
    /// String length compare (<=)
    #[clap(name = "len-le")]
    StringLenLe { string: String, n: usize },
    /// String length compare (=)
    #[clap(name = "len-eq")]
    StringLenEq { string: String, n: usize },
    /// Advise quoting if a value looks like an unquoted shell word that may be misinterpreted
    #[clap(name = "advise-quote")]
    AdviseQuote { value: String },
}

#[derive(Subcommand)]
enum NumberCommand {
    /// Checks if two numbers are equal (-eq).
    #[clap(name = "eq")]
    NumberEqual { num1: i64, num2: i64 },
    /// Checks if two numbers are not equal (-ne).
    #[clap(name = "ne")]
    NumberNotEqual { num1: i64, num2: i64 },
    /// Checks if the first number is greater than the second (-gt).
    #[clap(name = "gt")]
    GreaterThan { num1: i64, num2: i64 },
    /// Checks if the first number is greater than or equal to the second (-ge).
    #[clap(name = "ge")]
    GreaterThanOrEqual { num1: i64, num2: i64 },
    /// Checks if the first number is less than the second (-lt).
    #[clap(name = "lt")]
    LessThan { num1: i64, num2: i64 },
    /// Checks if the first number is less than or equal to the second (-le).
    #[clap(name = "le")]
    LessThanOrEqual { num1: i64, num2: i64 },
    /// Integer in inclusive range [min, max]
    #[clap(name = "in-range")]
    InRangeInt { value: i64, min: i64, max: i64 },
    /// Number is positive (> 0)
    #[clap(name = "positive")]
    NumberIsPositive { n: f64 },
    /// Number is negative (< 0)
    #[clap(name = "negative")]
    NumberIsNegative { n: f64 },
}

#[derive(Subcommand)]
enum FloatCommand {
    /// Float in inclusive range [min, max]
    #[clap(name = "in-range")]
    InRangeFloat { min: f64, max: f64, value: f64 },
    /// Float comparisons: equal
    #[clap(name = "eq")]
    FloatEq { num1: f64, num2: f64 },
    /// Float comparisons: not equal
    #[clap(name = "ne")]
    FloatNe { num1: f64, num2: f64 },
    /// Float comparisons: greater than
    #[clap(name = "gt")]
    FloatGt { num1: f64, num2: f64 },
    /// Float comparisons: greater than or equal
    #[clap(name = "ge")]
    FloatGe { num1: f64, num2: f64 },
    /// Float comparisons: less than
    #[clap(name = "lt")]
    FloatLt { num1: f64, num2: f64 },
    /// Float comparisons: less than or equal
    #[clap(name = "le")]
    FloatLe { num1: f64, num2: f64 },
    /// Float approximately equal within epsilon
    #[clap(name = "approx-eq")]
    FloatApproxEq { a: f64, b: f64, epsilon: f64 },
}

#[derive(Subcommand)]
enum SemverCommand {
    /// Semantic version compare equal
    #[clap(name = "eq")]
    SemverEq { v1: String, v2: String },
    /// Semantic version compare not equal
    #[clap(name = "ne")]
    SemverNe { v1: String, v2: String },
    /// Semantic version greater than
    #[clap(name = "gt")]
    SemverGt { v1: String, v2: String },
    /// Semantic version greater than or equal
    #[clap(name = "ge")]
    SemverGe { v1: String, v2: String },
    /// Semantic version less than
    #[clap(name = "lt")]
    SemverLt { v1: String, v2: String },
    /// Semantic version less than or equal
    #[clap(name = "le")]
    SemverLe { v1: String, v2: String },
}

#[derive(Subcommand)]
enum EnvCommand {
    /// Check if environment variable is set and non-empty
    #[clap(name = "set")]
    EnvSet { name: String },
    /// Environment variable equals value
    #[clap(name = "equal-to")]
    EnvEquals { name: String, value: String },
}

#[derive(Subcommand)]
enum NetCommand {
    /// Check whether we can reach the internet (TCP connect 1.1.1.1:53)
    #[clap(name = "online")]
    Online {},
    /// Check if TCP port is open on host within optional timeout (ms)
    #[clap(name = "port-open")]
    NetPortOpen { host: String, port: u16, #[clap(long, default_value_t = 1000)] timeout_ms: u64 },
}

#[derive(Subcommand)]
enum SystemCommand {
    /// Detect operating system equals the given name (linux, macos, windows, freebsd, netbsd, openbsd, dragonfly, android, ios)
    #[clap(name = "os")]
    Os { name: String },
    /// Check if a command exists in PATH and is executable
    #[clap(name = "command-exists")]
    CommandExists { command: String },
    /// Architecture equals given name
    #[clap(name = "arch")]
    ArchIs { name: String },
    /// Checks if a file descriptor is open on a terminal (-t FD).
    #[clap(name = "fd-tty")]
    Tty { fd: i32 },
}

#[derive(Subcommand)]
enum Commands {
    /// File-related checks
    #[command(subcommand)]
    File(FileCommand),
    /// String-related checks
    #[command(subcommand)]
    String(StringCommand),
    /// Integer-related checks
    #[command(subcommand)]
    Int(NumberCommand),
    /// Floating point-related checks
    #[command(subcommand)]
    Float(FloatCommand),
    /// Semantic versioning-related checks
    #[command(subcommand)]
    Semver(SemverCommand),
    /// Environment variable-related checks
    #[command(subcommand)]
    Env(EnvCommand),
    /// Network-related checks
    #[command(subcommand)]
    Net(NetCommand),
    /// System-related checks
    #[command(subcommand)]
    System(SystemCommand),
}

fn expand_path(path_str: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(path_str).into_owned())
}

fn handle_file_check<F>(path: &str, check: F)
where
    F: FnOnce(&fs::Metadata) -> bool,
{
    let path = expand_path(path);
    if let Ok(metadata) = fs::metadata(&path) {
        if check(&metadata) {
            exit(0);
        }
    }
    exit(1);
}

fn check_access(path: &str, mode: i32) -> bool {
    let expanded = expand_path(path);
    let path_str = expanded.to_string_lossy();
    if let Ok(c_path) = CString::new(path_str.as_bytes()) {
        unsafe { libc::access(c_path.as_ptr(), mode) == 0 }
    } else {
        false
    }
}

fn path_is_executable(candidate: &Path) -> bool {
    let path_str = candidate.to_string_lossy();
    if let Ok(c_path) = CString::new(path_str.as_bytes()) {
        unsafe { libc::access(c_path.as_ptr(), libc::X_OK) == 0 }
    } else {
        false
    }
}

fn command_exists_on_path(command: &str) -> bool {
    let candidate = Path::new(command);
    if candidate.components().count() > 1 {
        return path_is_executable(candidate);
    }
    if let Some(paths_os) = env::var_os("PATH") {
        let paths = env::split_paths(&paths_os);
        for dir in paths {
            let exe_path = dir.join(command);
            if path_is_executable(&exe_path) {
                return true;
            }
        }
    }
    false
}

fn eq_ci(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b) || a.to_lowercase() == b.to_lowercase()
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::File(file_command) => match file_command {
            FileCommand::Exists { path } => {
                if expand_path(path).exists() {
                    exit(0);
                }
                exit(1);
            }
            FileCommand::Directory { path } => handle_file_check(path, |m| m.is_dir()),
            FileCommand::File { path } => handle_file_check(path, |m| m.is_file()),
            FileCommand::Symlink { path } => {
                if let Ok(metadata) = fs::symlink_metadata(expand_path(path)) {
                    if metadata.is_symlink() {
                        exit(0);
                    }
                }
                exit(1);
            }
            FileCommand::BlockDevice { path } => {
                handle_file_check(path, |m| m.file_type().is_block_device())
            }
            FileCommand::CharacterDevice { path } => {
                handle_file_check(path, |m| m.file_type().is_char_device())
            }
            FileCommand::NamedPipe { path } => handle_file_check(path, |m| m.file_type().is_fifo()),
            FileCommand::Socket { path } => handle_file_check(path, |m| m.file_type().is_socket()),
            FileCommand::NonEmpty { path } => handle_file_check(path, |m| m.len() > 0),
            FileCommand::Readable { path } => {
                if check_access(path, libc::R_OK) {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            FileCommand::Writable { path } => {
                if check_access(path, libc::W_OK) {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            FileCommand::Executable { path } => {
                if check_access(path, libc::X_OK) {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            FileCommand::Suid { path } => {
                handle_file_check(path, |m| m.permissions().mode() & 0o4000 != 0)
            }
            FileCommand::Sgid { path } => {
                handle_file_check(path, |m| m.permissions().mode() & 0o2000 != 0)
            }
            FileCommand::Sticky { path } => {
                handle_file_check(path, |m| m.permissions().mode() & 0o1000 != 0)
            }
            FileCommand::OwnedByEffectiveUser { path } => handle_file_check(path, |_m| {
                // We need raw metadata to access uid; use metadata again here
                let p = expand_path(path);
                if let Ok(meta) = fs::metadata(&p) {
                    let file_uid = meta.uid();
                    let euid = unsafe { libc::geteuid() };
                    file_uid == euid
                } else {
                    false
                }
            }),
            FileCommand::OwnedByEffectiveGroup { path } => handle_file_check(path, |_m| {
                let p = expand_path(path);
                if let Ok(meta) = fs::metadata(&p) {
                    let file_gid = meta.gid();
                    let egid = unsafe { libc::getegid() };
                    file_gid == egid
                } else {
                    false
                }
            }),
            FileCommand::SameInode { path1, path2 } => {
                let path1 = expand_path(path1);
                let path2 = expand_path(path2);
                if let (Ok(meta1), Ok(meta2)) = (fs::metadata(&path1), fs::metadata(&path2)) {
                    if meta1.dev() == meta2.dev() && meta1.ino() == meta2.ino() {
                        exit(0);
                    }
                }
                exit(1);
            }
            FileCommand::Newer { path1, path2 } => {
                let path1 = expand_path(path1);
                let path2 = expand_path(path2);
                if let (Ok(meta1), Ok(meta2)) = (fs::metadata(&path1), fs::metadata(&path2)) {
                    if let (Ok(time1), Ok(time2)) = (meta1.modified(), meta2.modified()) {
                        if time1 > time2 {
                            exit(0);
                        }
                    }
                }
                exit(1);
            }
            FileCommand::Older { path1, path2 } => {
                let path1 = expand_path(path1);
                let path2 = expand_path(path2);
                if let (Ok(meta1), Ok(meta2)) = (fs::metadata(&path1), fs::metadata(&path2)) {
                    if let (Ok(time1), Ok(time2)) = (meta1.modified(), meta2.modified()) {
                        if time1 < time2 {
                            exit(0);
                        }
                    }
                }
                exit(1);
            }
            FileCommand::ExistsGlob { pattern } => {
                let expanded = shellexpand::tilde(pattern).into_owned();
                match glob(&expanded) {
                    Ok(paths) => {
                        for entry in paths {
                            if let Ok(p) = entry { if p.exists() { exit(0); } }
                        }
                        exit(1);
                    }
                    Err(_) => exit(1),
                }
            }
            FileCommand::NonEmptyGlob { pattern } => {
                let expanded = shellexpand::tilde(pattern).into_owned();
                match glob(&expanded) {
                    Ok(paths) => {
                        for entry in paths {
                            if let Ok(p) = entry {
                                if let Ok(md) = fs::metadata(&p) { if md.len() > 0 { exit(0); } }
                            }
                        }
                        exit(1);
                    }
                    Err(_) => exit(1),
                }
            }
            FileCommand::FileSizeGt { path, bytes } => handle_file_check(path, |m| m.len() > *bytes),
            FileCommand::FileSizeGe { path, bytes } => handle_file_check(path, |m| m.len() >= *bytes),
            FileCommand::FileSizeLt { path, bytes } => handle_file_check(path, |m| m.len() < *bytes),
            FileCommand::FileSizeLe { path, bytes } => handle_file_check(path, |m| m.len() <= *bytes),
            FileCommand::FileSizeEq { path, bytes } => handle_file_check(path, |m| m.len() == *bytes),
            FileCommand::FileMtimeOlderThan { path, seconds } => {
                let path = expand_path(path);
                if let Ok(md) = fs::metadata(&path) {
                    if let Ok(modified) = md.modified() {
                        if let Ok(age) = modified.elapsed() {
                            if age.as_secs() > *seconds { exit(0); } else { exit(1); }
                        } else { exit(1); }
                    } else { exit(1); }
                } else { exit(1); }
            }
            FileCommand::FileMtimeNewerThan { path, seconds } => {
                let path = expand_path(path);
                if let Ok(md) = fs::metadata(&path) {
                    if let Ok(modified) = md.modified() {
                        if let Ok(age) = modified.elapsed() {
                            if age.as_secs() < *seconds { exit(0); } else { exit(1); }
                        } else { exit(1); }
                    } else { exit(1); }
                } else { exit(1); }
            }
        },
        Commands::String(string_command) => match string_command {
            StringCommand::Equal { string1, string2 } => {
                if string1 == string2 {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            StringCommand::NotEqual { string1, string2 } => {
                if string1 != string2 {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            StringCommand::EmptyString { string } => {
                if string.is_empty() {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            StringCommand::NonEmptyString { string } => {
                if !string.is_empty() {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            StringCommand::EqualCaseInsensitive { string1, string2 } => {
                if eq_ci(string1, string2) { exit(0); } else { exit(1); }
            }
            StringCommand::Regex { string, pattern } => {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(string) { exit(0); } else { exit(1); }
                } else { exit(1); }
            }
            StringCommand::RegexCaseInsensitive { string, pattern } => {
                let pat = format!("(?i:{})", pattern);
                if let Ok(re) = Regex::new(&pat) {
                    if re.is_match(string) { exit(0); } else { exit(1); }
                } else { exit(1); }
            }
            StringCommand::Contains { string, needle } => {
                if string.contains(needle) { exit(0); } else { exit(1); }
            }
            StringCommand::ContainsCaseInsensitive { string, needle } => {
                if string.to_lowercase().contains(&needle.to_lowercase()) { exit(0); } else { exit(1); }
            }
            StringCommand::StartsWith { string, prefix } => {
                if string.starts_with(prefix) { exit(0); } else { exit(1); }
            }
            StringCommand::StartsWithCaseInsensitive { string, prefix } => {
                if string.to_lowercase().starts_with(&prefix.to_lowercase()) { exit(0); } else { exit(1); }
            }
            StringCommand::EndsWith { string, suffix } => {
                if string.ends_with(suffix) { exit(0); } else { exit(1); }
            }
            StringCommand::EndsWithCaseInsensitive { string, suffix } => {
                if string.to_lowercase().ends_with(&suffix.to_lowercase()) { exit(0); } else { exit(1); }
            }
            StringCommand::IsInteger { string } => {
                if string.parse::<i64>().is_ok() { exit(0); } else { exit(1); }
            }
            StringCommand::IsNumber { string } => {
                if string.parse::<f64>().is_ok() { exit(0); } else { exit(1); }
            }
            StringCommand::StringIsUuid { string } => {
                let pat = Regex::new(r"(?i)^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();
                if pat.is_match(string) { exit(0); } else { exit(1); }
            }
            StringCommand::StringIsIpv4 { string } => {
                if string.parse::<Ipv4Addr>().is_ok() { exit(0); } else { exit(1); }
            }
            StringCommand::StringAsciiOnly { string } => {
                if string.chars().all(|c| c.is_ascii()) { exit(0); } else { exit(1); }
            }
            StringCommand::StringLenGt { string, n } => { if string.chars().count() > *n { exit(0); } else { exit(1); } }
            StringCommand::StringLenGe { string, n } => { if string.chars().count() >= *n { exit(0); } else { exit(1); } }
            StringCommand::StringLenLt { string, n } => { if string.chars().count() < *n { exit(0); } else { exit(1); } }
            StringCommand::StringLenLe { string, n } => { if string.chars().count() <= *n { exit(0); } else { exit(1); } }
            StringCommand::StringLenEq { string, n } => { if string.chars().count() == *n { exit(0); } else { exit(1); } }
            StringCommand::AdviseQuote { value } => {
                let suspicious = value.is_empty()
                    || value.starts_with('-')
                    || matches!(value.as_str(), "-a"|"-o"|"!"|"("|")");
                if suspicious {
                    eprintln!("Value '{}' may need quoting. Consider using \"$VAR\" in your shell.", value);
                    exit(1);
                } else {
                    exit(0);
                }
            }
        },
        Commands::Int(number_command) => match number_command {
            NumberCommand::NumberEqual { num1, num2 } => {
                if num1 == num2 {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            NumberCommand::NumberNotEqual { num1, num2 } => {
                if num1 != num2 {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            NumberCommand::GreaterThan { num1, num2 } => {
                if num1 > num2 {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            NumberCommand::GreaterThanOrEqual { num1, num2 } => {
                if num1 >= num2 {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            NumberCommand::LessThan { num1, num2 } => {
                if num1 < num2 {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            NumberCommand::LessThanOrEqual { num1, num2 } => {
                if num1 <= num2 {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            NumberCommand::InRangeInt { value, min, max } => {
                if value >= min && value <= max { exit(0); } else { exit(1); }
            }
            NumberCommand::NumberIsPositive { n } => { if *n > 0.0 { exit(0); } else { exit(1); } }
            NumberCommand::NumberIsNegative { n } => { if *n < 0.0 { exit(0); } else { exit(1); } }
        },
        Commands::Float(float_command) => match float_command {
            FloatCommand::InRangeFloat { min, max, value } => {
                if value >= min && value <= max { exit(0); } else { exit(1); }
            }
            FloatCommand::FloatEq { num1, num2 } => {
                if (num1 - num2).abs() == 0.0 { exit(0); } else { exit(1); }
            }
            FloatCommand::FloatNe { num1, num2 } => {
                if (num1 - num2).abs() != 0.0 { exit(0); } else { exit(1); }
            }
            FloatCommand::FloatGt { num1, num2 } => { if num1 > num2 { exit(0); } else { exit(1); } }
            FloatCommand::FloatGe { num1, num2 } => { if num1 >= num2 { exit(0); } else { exit(1); } }
            FloatCommand::FloatLt { num1, num2 } => { if num1 < num2 { exit(0); } else { exit(1); } }
            FloatCommand::FloatLe { num1, num2 } => { if num1 <= num2 { exit(0); } else { exit(1); } }
            FloatCommand::FloatApproxEq { a, b, epsilon } => {
                if (*a - *b).abs() <= *epsilon { exit(0); } else { exit(1); }
            }
        },
        Commands::Semver(semver_command) => match semver_command {
            SemverCommand::SemverEq { v1, v2 } => {
                if let (Ok(a), Ok(b)) = (Version::parse(v1), Version::parse(v2)) {
                    if a == b { exit(0); } else { exit(1); }
                } else { exit(1); }
            }
            SemverCommand::SemverNe { v1, v2 } => {
                if let (Ok(a), Ok(b)) = (Version::parse(v1), Version::parse(v2)) {
                    if a != b { exit(0); } else { exit(1); }
                } else { exit(1); }
            }
            SemverCommand::SemverGt { v1, v2 } => {
                if let (Ok(a), Ok(b)) = (Version::parse(v1), Version::parse(v2)) {
                    if a > b { exit(0); } else { exit(1); }
                } else { exit(1); }
            }
            SemverCommand::SemverGe { v1, v2 } => {
                if let (Ok(a), Ok(b)) = (Version::parse(v1), Version::parse(v2)) {
                    if a >= b { exit(0); } else { exit(1); }
                } else { exit(1); }
            }
            SemverCommand::SemverLt { v1, v2 } => {
                if let (Ok(a), Ok(b)) = (Version::parse(v1), Version::parse(v2)) {
                    if a < b { exit(0); } else { exit(1); }
                } else { exit(1); }
            }
            SemverCommand::SemverLe { v1, v2 } => {
                if let (Ok(a), Ok(b)) = (Version::parse(v1), Version::parse(v2)) {
                    if a <= b { exit(0); } else { exit(1); }
                } else { exit(1); }
            }
        },
        Commands::Env(env_command) => match env_command {
            EnvCommand::EnvSet { name } => {
                match env::var_os(name) {
                    Some(val) => {
                        if !val.is_empty() { exit(0); } else { exit(1); }
                    }
                    None => exit(1)
                }
            }
            EnvCommand::EnvEquals { name, value } => {
                match env::var(name) {
                    Ok(v) => if &v == value { exit(0); } else { exit(1); },
                    Err(_) => exit(1),
                }
            }
        },
        Commands::Net(net_command) => match net_command {
            NetCommand::Online {} => {
                let addr = "1.1.1.1:53";
                match TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_millis(800)) {
                    Ok(_) => exit(0),
                    Err(_) => exit(1),
                }
            }
            NetCommand::NetPortOpen { host, port, timeout_ms } => {
                let addr = format!("{}:{}", host, port);
                let timeout = Duration::from_millis(*timeout_ms);
                match addr.parse() {
                    Ok(sockaddr) => match TcpStream::connect_timeout(&sockaddr, timeout) {
                        Ok(_) => exit(0),
                        Err(_) => exit(1),
                    },
                    Err(_) => exit(1),
                }
            }
        },
        Commands::System(system_command) => match system_command {
            SystemCommand::Os { name } => {
                let os = env::consts::OS; // e.g., "linux", "macos", "windows"
                if eq_ci(os, name) {
                    exit(0);
                } else {
                    exit(1);
                }
            }
            SystemCommand::CommandExists { command } => {
                if command_exists_on_path(command) { exit(0); } else { exit(1); }
            }
            SystemCommand::ArchIs { name } => {
                if eq_ci(env::consts::ARCH, name) { exit(0); } else { exit(1); }
            }
            SystemCommand::Tty { fd } => {
                let is_tty = unsafe { libc::isatty(*fd) == 1 };
                if is_tty {
                    exit(0);
                } else {
                    exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_expand_path_with_tilde() {
        // This test assumes a typical home directory setup.
        // It might fail in unusual environments.
        let home = env::var("HOME").unwrap();
        assert_eq!(expand_path("~/test"), PathBuf::from(format!("{}/test", home)));
    }

    #[test]
    fn test_expand_path_without_tilde() {
        assert_eq!(expand_path("/tmp/test"), PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_check_access_readable() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("readable.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "content").unwrap();

        let path_str = file_path.to_str().unwrap();
        assert!(check_access(path_str, libc::R_OK));
    }

    #[test]
    fn test_check_access_writable() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("writable.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "content").unwrap();

        let path_str = file_path.to_str().unwrap();
        assert!(check_access(path_str, libc::W_OK));
    }

    #[test]
    fn test_path_is_executable() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("executable_script");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "#!/bin/sh\necho hello").unwrap();

        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&file_path, perms).unwrap();

        assert!(path_is_executable(&file_path));
    }
    
    #[test]
    fn test_command_exists_on_path_positive() {
        // This test assumes 'ls' is available on the system PATH.
        assert!(command_exists_on_path("ls"));
    }

    #[test]
    fn test_command_exists_on_path_negative() {
        assert!(!command_exists_on_path("non_existent_command_1234567890"));
    }

    #[test]
    fn test_eq_ci() {
        assert!(eq_ci("hello", "HELLO"));
        assert!(eq_ci("Test", "test"));
        assert!(!eq_ci("hello", "world"));
    }
}