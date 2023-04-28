use anyhow::{bail, Context, Ok, Result};
use clap::{Arg, Command};
use colored::*;
use std::env;
#[allow(unused_imports)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[allow(dead_code)]
enum Mode {
    Executable,
    File,
    NotFounded,
}

fn filename_to_string(filename: PathBuf) -> Result<String> {
    Ok(filename
        .to_str()
        .with_context(|| format!("Failed to convert {} to a valid string", filename.display()))?
        .to_string())
}

fn local_print(_i: usize, path: PathBuf, arg: &String, mode: Mode, found: bool) {
    match mode {
        Mode::Executable => println!(
            "{}",
            format!(
                "* {} {}",
                arg,
                if !found {
                    filename_to_string(path).unwrap().green().bold()
                } else {
                    filename_to_string(path).unwrap().bold()
                }
            )
        ),
        Mode::File => println!(
            "{}",
            format!(
                "- {} {}",
                arg,
                if !found {
                    filename_to_string(path).unwrap().green().bold()
                } else {
                    filename_to_string(path).unwrap().bold()
                }
            )
        ),
        Mode::NotFounded => println!("{}", format!("x {arg} not found in PATH!").red().bold()),
    }
}

fn main() -> Result<()> {
    let which = Command::new("which")
        .author("Tomzz, pythongolangkotlinrust@gmail.com")
        .version("1.0.0")
        .about("Search in the path for a specific executable file")
        .arg(
            Arg::new("filenames")
                .required(true)
                .index(1)
                .help("The filenames to search for"),
        )
        .after_help("To be improved")
        .get_matches();

    // let mut args: Vec<String> = env::args().skip(1).collect();
    let mut args: Vec<String> = which.get_many("filenames").unwrap().cloned().collect();
    if args.is_empty() {
        bail!("not enough arguments");
        // println!("not enough arguments!");
        // std::process::exit(1);
    }
    match env::consts::OS {
        "linux" | "macos" => {
            for i in 0..args.len() {
                let mut filename = PathBuf::from(&args[i]);
                if let Some(ext) = filename.extension() {
                    if ext == "exe" {
                        filename.set_extension("");
                        args[i] = filename_to_string(filename)?;
                    }
                }
            }
        }
        "windows" => {
            for i in 0..args.len() {
                let mut filename = PathBuf::from(&args[i]);
                if let Some(ext) = filename.extension() {
                    if ext != "exe" {
                        filename.set_extension("exe");
                        args[i] = filename_to_string(filename)?;
                    }
                } else {
                    filename.set_extension("exe");
                    args[i] = filename_to_string(filename)?;
                }
            }
        }
        _ => (),
    }
    // println!("processed arguments: {args:?}");
    if let Some(paths) = env::var_os("PATH") {
        let paths: Vec<String> = env::split_paths(&paths)
            .map(|path| path.to_string_lossy().into_owned())
            .collect();

        for (i, arg) in args.iter().enumerate() {
            let mut found = false;
            for path in &paths {
                let full_path = PathBuf::from(path).join(arg);
                // TODO judge whether the file is executable
                if full_path.exists() && full_path.is_file() {
                    #[cfg(unix)]
                    {
                        if fs::metadata(&full_path)?.permissions().mode() & 0o111 != 0 {
                            local_print(i + 1, full_path, arg, Mode::Executable, found);
                        } else {
                            local_print(i + 1, full_path, arg, Mode::File, found);
                        }
                    }
                    #[cfg(windows)]
                    {
                        local_print(i + 1, full_path, arg, Mode::Executable, found);
                    }
                    if !found {
                        found = true;
                    }
                }
            }
            if !found {
                local_print(i + 1, PathBuf::new(), arg, Mode::NotFounded, found);
            }
        }
    } else {
        bail!("PATH environment variable is not defined!");
        // println!("PATH environment variable is not defined!");
        // std::process::exit(1);
    }
    Ok(())
}
