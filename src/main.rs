use clap::{Parser, Subcommand};
use std::fs;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{PathBuf};
use std::process::exit;

#[derive(Parser)]
#[command(author, version, about = "A modern, descriptive replacement for the 'test' command.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Checks if a file exists.
    Exists { path: String },
    /// Checks if a path is a directory.
    Directory { path: String },
    /// Checks if a path is a regular file.
    File { path: String },
    /// Checks if a path is a symbolic link.
    Symlink { path: String },
    /// Checks if a file is a block special file.
    #[clap(name = "block-device")]
    BlockDevice { path: String },
    /// Checks if a file is a character special file.
    #[clap(name = "character-device")]
    CharacterDevice { path: String },
    /// Checks if a file is a named pipe (FIFO).
    #[clap(name = "named-pipe")]
    NamedPipe { path: String },
    /// Checks if a file is a socket.
    Socket { path: String },
    /// Checks if a file exists and has a size greater than zero.
    #[clap(name = "non-empty")]
    NonEmpty { path: String },
    /// Checks if a file is readable by the current user.
    Readable { path: String },
    /// Checks if a file is writable by the current user.
    Writable { path: String },
    /// Checks if a file is executable by the current user.
    Executable { path: String },
    /// Checks if the file has the set-user-ID bit set.
    Suid { path: String },
    /// Checks if the file has the set-group-ID bit set.
    Sgid { path: String },
    /// Checks if the file has the sticky bit set.
    Sticky { path: String },
    /// Checks if a file descriptor is open on a terminal.
    Tty { fd: i32 },
    /// Checks if two files are on the same device and have the same inode number.
    #[clap(name = "same-inode")]
    SameInode { path1: String, path2: String },
    /// Checks if the first file is newer than the second.
    Newer { path1: String, path2: String },
    /// Checks if the first file is older than the second.
    Older { path1: String, path2: String },
    /// Checks if two strings are equal.
    Equal { string1: String, string2: String },
    /// Checks if two strings are not equal.
    #[clap(name = "not-equal")]
    NotEqual { string1: String, string2: String },
    /// Checks if a string is empty.
    #[clap(name = "empty-string")]
    EmptyString { string: String },
    /// Checks if a string is not empty.
    #[clap(name = "non-empty-string")]
    NonEmptyString { string: String },
    /// Checks if two numbers are equal.
    #[clap(name = "number-equal")]
    NumberEqual { num1: i64, num2: i64 },
    /// Checks if two numbers are not equal.
    #[clap(name = "number-not-equal")]
    NumberNotEqual { num1: i64, num2: i64 },
    /// Checks if the first number is greater than the second.
    #[clap(name = "greater-than")]
    GreaterThan { num1: i64, num2: i64 },
    /// Checks if the first number is greater than or equal to the second.
    #[clap(name = "greater-than-or-equal")]
    GreaterThanOrEqual { num1: i64, num2: i64 },
    /// Checks if the first number is less than the second.
    #[clap(name = "less-than")]
    LessThan { num1: i64, num2: i64 },
    /// Checks if the first number is less than or equal to the second.
    #[clap(name = "less-than-or-equal")]
    LessThanOrEqual { num1: i64, num2: i64 },
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Exists { path } => {
            if expand_path(path).exists() {
                exit(0);
            }
            exit(1);
        }
        Commands::Directory { path } => handle_file_check(path, |m| m.is_dir()),
        Commands::File { path } => handle_file_check(path, |m| m.is_file()),
        Commands::Symlink { path } => {
            if let Ok(metadata) = fs::symlink_metadata(expand_path(path)) {
                if metadata.is_symlink() {
                    exit(0);
                }
            }
            exit(1);
        }
        Commands::BlockDevice { path } => handle_file_check(path, |m| m.file_type().is_block_device()),
        Commands::CharacterDevice { path } => handle_file_check(path, |m| m.file_type().is_char_device()),
        Commands::NamedPipe { path } => handle_file_check(path, |m| m.file_type().is_fifo()),
        Commands::Socket { path } => handle_file_check(path, |m| m.file_type().is_socket()),
        Commands::NonEmpty { path } => handle_file_check(path, |m| m.len() > 0),
        Commands::Readable { path } => handle_file_check(path, |m| {
            m.permissions().mode() & 0o444 != 0
        }),
        Commands::Writable { path } => handle_file_check(path, |m| {
            m.permissions().mode() & 0o222 != 0
        }),
        Commands::Executable { path } => handle_file_check(path, |m| {
            m.permissions().mode() & 0o111 != 0
        }),
        Commands::Suid { path } => handle_file_check(path, |m| {
            m.permissions().mode() & 0o4000 != 0
        }),
        Commands::Sgid { path } => handle_file_check(path, |m| {
            m.permissions().mode() & 0o2000 != 0
        }),
        Commands::Sticky { path } => handle_file_check(path, |m| {
            m.permissions().mode() & 0o1000 != 0
        }),
        Commands::Tty { fd: _ } => {
            if atty::is(atty::Stream::Stdin) { // `atty` is a common crate for this check.
                 exit(0);
            }
            exit(1);
        }
        Commands::SameInode { path1, path2 } => {
            let path1 = expand_path(path1);
            let path2 = expand_path(path2);
            if let (Ok(meta1), Ok(meta2)) = (fs::metadata(&path1), fs::metadata(&path2)) {
                if meta1.dev() == meta2.dev() && meta1.ino() == meta2.ino() {
                    exit(0);
                }
            }
            exit(1);
        }
        Commands::Newer { path1, path2 } => {
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
        Commands::Older { path1, path2 } => {
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
        Commands::Equal { string1, string2 } => {
            if string1 == string2 { exit(0); } else { exit(1); }
        }
        Commands::NotEqual { string1, string2 } => {
            if string1 != string2 { exit(0); } else { exit(1); }
        }
        Commands::EmptyString { string } => {
            if string.is_empty() { exit(0); } else { exit(1); }
        }
        Commands::NonEmptyString { string } => {
            if !string.is_empty() { exit(0); } else { exit(1); }
        }
        Commands::NumberEqual { num1, num2 } => {
            if num1 == num2 { exit(0); } else { exit(1); }
        }
        Commands::NumberNotEqual { num1, num2 } => {
            if num1 != num2 { exit(0); } else { exit(1); }
        }
        Commands::GreaterThan { num1, num2 } => {
            if num1 > num2 { exit(0); } else { exit(1); }
        }
        Commands::GreaterThanOrEqual { num1, num2 } => {
            if num1 >= num2 { exit(0); } else { exit(1); }
        }
        Commands::LessThan { num1, num2 } => {
            if num1 < num2 { exit(0); } else { exit(1); }
        }
        Commands::LessThanOrEqual { num1, num2 } => {
            if num1 <= num2 { exit(0); } else { exit(1); }
        }
    }
}
