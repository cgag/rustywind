use async_std::fs;
use async_std::task;
use clap::{App, AppSettings, Arg};
use futures::stream::{FuturesUnordered, StreamExt};
use indoc::indoc;
use rayon::prelude::*;
use rustywind::options::{Options, WriteMode};
use std::path::Path;
use std::path::PathBuf;

#[async_std::main]
async fn main() {
    let matches = App::new("RustyWind")
        .version(clap::crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .author("Praveen Perera <praveen@avencera.com>")
        .about("\nOrganize all your tailwind classes")
        .usage(indoc!("
        Run rustywind with a path to get a list of files that will be changed
              rustywind . --dry-run

            If you want to reorganize all classes in place, and change the files run with the `--write` flag
              rustywind --write .
                         
            rustywind [FLAGS] <PATH>"))
        .arg(
            Arg::with_name("file_or_dir")
                .value_name("PATH")
                .help("A file or directory to run on")
                .index(1)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("write")
                .long("write")
                .conflicts_with("dry-run")
                .help("Changes the files in place with the reorganized classes"),
        )
        .arg(
            Arg::with_name("dry_run")
                .long("dry-run")
                .conflicts_with("write")
                .help("Prints out the new file content with the sorted classes to the terminal"),
        )
        .arg(
            Arg::with_name("allow-duplicates")
                .long("allow-duplicates")
                .help("When set, rustywind will not delete duplicated classes"),
        )
        .get_matches();

    let options = Options::new_from_matches(&matches);

    match &options.write_mode {
        WriteMode::DryRun => println!(
            "\ndry run mode activated: here is a list of files that \
             would be changed when you run with the --write flag"
        ),

        WriteMode::ToFile => {
            println!("\nwrite mode is active the following files are being saved:")
        }

        WriteMode::ToConsole => println!(
            "\nprinting file contents to console, run with --write to save changes to files:"
        ),
    }
    options
        .search_paths
        .par_iter()
        .map(|&file_path| async {
            run_on_file_paths(&file_path, &options).await;
        })
        .collect::<Vec<_>>()
}

async fn run_on_file_paths(file_path: Path, options: &Options) {
    match fs::read_to_string(file_path).await {
        Ok(contents) => {
            if rustywind::has_classes(&contents) {
                let sorted_content = rustywind::sort_file_contents(contents, options);

                match &options.write_mode {
                    WriteMode::DryRun => print_file_name(file_path, options),
                    WriteMode::ToFile => write_to_file(file_path, &sorted_content, options).await,
                    WriteMode::ToConsole => print_file_contents(&sorted_content),
                }
            }
        }
        Err(_error) => (),
    }
}

async fn write_to_file(file_path: &Path, sorted_contents: &str, options: &Options) {
    match fs::write(file_path, sorted_contents.as_bytes()).await {
        Ok(_) => print_file_name(file_path, options),
        Err(err) => {
            eprintln!("\nError: {:?}", err);
            eprintln!(
                "Unable to to save file: {}",
                get_file_name(file_path, &options.starting_path)
            );
        }
    }
}

fn print_file_name(file_path: &Path, options: &Options) {
    println!("  * {}", get_file_name(file_path, &options.starting_path))
}

fn get_file_name(file_path: &Path, dir: &Path) -> String {
    file_path
        .strip_prefix(dir)
        .unwrap_or(file_path)
        .display()
        .to_string()
}

fn print_file_contents(file_contents: &str) {
    println!("\n\n{}\n\n", file_contents)
}
