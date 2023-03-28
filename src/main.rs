use glob::Pattern;
use walkdir::WalkDir;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::vec::Vec;

fn get_ignore_list(ignore_file_path: &Path) -> Vec<String> {
    let mut ignore_list = Vec::new();
    if let Ok(content) = fs::read_to_string(ignore_file_path) {
        for line in content.lines() {
            let line = if cfg!(target_os = "windows") {
                line.replace("/", "\\")
            } else {
                line.to_string()
            };
            ignore_list.push(line);
        }
    }
    ignore_list
}

fn should_ignore(file_path: &Path, ignore_list: &[String]) -> bool {
    ignore_list.iter().any(|pattern| {
        Pattern::new(pattern)
            .map(|p| p.matches(file_path.to_str().unwrap_or("")))
            .unwrap_or(false)
    })
}

fn process_repository(
    repo_path: &Path,
    ignore_list: &[String],
    output_file: &mut fs::File,
) -> std::io::Result<()> {

    for entry in WalkDir::new(repo_path) {
        let entry = entry?;
        let file_path = entry.path();
        let relative_file_path = file_path.strip_prefix(repo_path).unwrap_or(&file_path);

        let file_type = entry.file_type();

        if file_type.is_file() {
            if !should_ignore(relative_file_path, ignore_list) {
                let mut file = fs::File::open(&file_path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                writeln!(output_file, "----")?;
                writeln!(output_file, "{}", relative_file_path.display())?;
                writeln!(output_file, "{}", contents)?;
            }
        }

    }
    writeln!(output_file, "--END--")?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: git_to_text /path/to/git/repository [-p /path/to/preamble.txt] [-o /path/to/output_file.txt]");
        std::process::exit(1);
    }

    let repo_path = PathBuf::from(&args[1]);
    let ignore_file_path = repo_path.join(".gptignore");
    let ignore_file_path = if cfg!(target_os = "windows") {
        ignore_file_path.to_str().unwrap().replace("/", "\\")
    } else {
        ignore_file_path.to_str().unwrap().to_string()
    };

    let ignore_list = get_ignore_list(&Path::new(&ignore_file_path));
    let preamble_file = args
        .iter()
        .position(|x| x == "-p")
        .map(|i| args[i + 1].clone());
    let output_file_path = args
        .iter()
        .position(|x| x == "-o")
        .map(|i| PathBuf::from(&args[i + 1]))
        .unwrap_or_else(|| PathBuf::from("output.txt"));

    let mut output_file = fs::File::create(&output_file_path)?;
    if let Some(preamble_file) = preamble_file {
        let preamble_text = fs::read_to_string(&preamble_file)?;
        writeln!(output_file, "{}", preamble_text)?;
    } else {
        writeln!(output_file, "The following text is a Git repository with code. The structure of the text are sections that begin with ----, followed by a single line containing the file path and file name, followed by a variable amount of lines containing the file contents. The text representing the Git repository ends when the symbols --END-- are encounted. Any further text beyond --END-- are meant to be interpreted as instructions using the aforementioned Git repository as context.\n")?;
        return process_repository(&repo_path, &ignore_list, &mut output_file);
    }

    return Ok(());
}
