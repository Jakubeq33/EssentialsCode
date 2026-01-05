use crate::fixer;
use crate::parser::Language;
use crate::ui;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub fn scan_project(path: &Path, lang: Option<&str>) -> Result<()> {
    ui::print_section("Scanning Project");
    
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let path_str = path.to_string_lossy().to_string();
    let path_str = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);
    let path = PathBuf::from(path_str);

    ui::print_info(&format!("Path: {}", path.display()));
    
    let languages = match lang {
        Some(l) => vec![detect_language_from_str(l)],
        None => detect_languages(&path),
    };

    if languages.is_empty() {
        ui::print_warning("No supported source files found");
        ui::print_hint("Supported: C++, Python, JavaScript, TypeScript, Rust");
        return Ok(());
    }

    ui::print_info(&format!(
        "Languages: {}",
        languages
            .iter()
            .map(|l| format!("{}", l))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    println!();

    let mut total_errors = 0;

    for lang in &languages {
        let errors = check_language(&path, lang)?;
        total_errors += errors;
    }

    if total_errors == 0 {
        ui::print_no_errors();
    } else {
        ui::print_errors_found(total_errors);
    }

    Ok(())
}

fn detect_language_from_str(s: &str) -> Language {
    match s.to_lowercase().as_str() {
        "cpp" | "c++" | "c" => Language::Cpp,
        "python" | "py" => Language::Python,
        "javascript" | "js" => Language::JavaScript,
        "typescript" | "ts" => Language::TypeScript,
        "rust" | "rs" => Language::Rust,
        _ => Language::Unknown,
    }
}

fn detect_languages(path: &Path) -> Vec<Language> {
    let mut langs = Vec::new();

    for entry in WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Some(ext) = entry.path().extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            let lang = match ext.as_str() {
                "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" => Some(Language::Cpp),
                "py" => Some(Language::Python),
                "js" | "jsx" | "mjs" => Some(Language::JavaScript),
                "ts" | "tsx" => Some(Language::TypeScript),
                "rs" => Some(Language::Rust),
                _ => None,
            };

            if let Some(l) = lang {
                if !langs.contains(&l) {
                    langs.push(l);
                }
            }
        }
    }

    langs
}

fn check_language(path: &Path, lang: &Language) -> Result<usize> {
    match lang {
        Language::Cpp => check_cpp(path),
        Language::Python => check_python(path),
        Language::JavaScript => check_javascript(path),
        Language::TypeScript => check_typescript(path),
        Language::Rust => check_rust(path),
        Language::Unknown => Ok(0),
    }
}

fn check_cpp(path: &Path) -> Result<usize> {
    let mut error_count = 0;

    let files: Vec<_> = WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| {
                    let ext = ext.to_string_lossy().to_lowercase();
                    matches!(ext.as_str(), "cpp" | "cc" | "cxx" | "c")
                })
                .unwrap_or(false)
        })
        .collect();

    for entry in files {
        let file_path = entry.path();

        let output = Command::new("g++")
            .args([
                "-std=c++17",
                "-Wall",
                "-fsyntax-only",
                file_path.to_str().unwrap_or(""),
            ])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => {
                Command::new("clang++")
                    .args([
                        "-std=c++17",
                        "-Wall",
                        "-fsyntax-only",
                        file_path.to_str().unwrap_or(""),
                    ])
                    .output()?
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error_count += process_compiler_errors(&stderr)?;
        }
    }

    Ok(error_count)
}

fn check_python(path: &Path) -> Result<usize> {
    let mut error_count = 0;

    let files: Vec<_> = WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.to_string_lossy().to_lowercase() == "py")
                .unwrap_or(false)
        })
        .filter(|e| {
            let path_str = e.path().to_string_lossy();
            !path_str.contains("__pycache__")
                && !path_str.contains(".venv")
                && !path_str.contains("venv")
                && !path_str.contains("node_modules")
                && !path_str.contains(".git")
        })
        .collect();

    for entry in &files {
        let file_path = entry.path();
        ui::print_info(&format!("Checking: {}", file_path.display()));
        
        let syntax_output = Command::new("python")
            .args(["-m", "py_compile", file_path.to_str().unwrap_or("")])
            .output();

        if let Ok(output) = syntax_output {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                ui::print_error("Syntax Error:");
                error_count += process_python_error(&stderr)?;
                continue;
            }
        }
        
        let run_output = Command::new("python")
            .arg(file_path.to_str().unwrap_or(""))
            .current_dir(path)
            .output();

        if let Ok(output) = run_output {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    error_count += process_python_error(&stderr)?;
                }
            }
        }
        
        let pylint_output = Command::new("python")
            .args([
                "-m", "pylint",
                "--errors-only",
                "--disable=import-error",
                file_path.to_str().unwrap_or(""),
            ])
            .output();

        if let Ok(output) = pylint_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() && stdout.contains(": E") {
                for line in stdout.lines() {
                    if line.contains(": E") {
                        ui::print_warning(&format!("Pylint: {}", line));
                        error_count += 1;
                    }
                }
            }
        }
    }
    
    for entry in &files {
        let file_path = entry.path();
        error_count += analyze_python_file(file_path)?;
    }

    Ok(error_count)
}

fn analyze_python_file(path: &Path) -> Result<usize> {
    let content = std::fs::read_to_string(path)?;
    let mut issues = 0;
    
    let patterns = [
        ("os.getenv(", "Possible None value from getenv - check if variable exists"),
        (".get(\"", "Dictionary .get() may return None - handle None case"),
        ("r.json()[", "Direct JSON access may raise KeyError - use .get()"),
        ("data[\"", "Direct dict access may raise KeyError if key missing"),
        (".lower()", "Calling .lower() on possibly None value"),
        (".upper()", "Calling .upper() on possibly None value"),
        ("datetime.fromisoformat(", "fromisoformat() will fail on None or invalid string"),
    ];

    for (pattern, warning) in patterns {
        if content.contains(pattern) {
            let line_num = content
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains(pattern))
                .map(|(i, _)| i + 1)
                .unwrap_or(0);

            if line_num > 0 {
                ui::print_warning(&format!(
                    "{}:{} - {}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    line_num,
                    warning
                ));
                issues += 1;
            }
        }
    }
    
    if content.contains("f\"") && content.contains("os.getenv") {
        if content.contains("http") || content.contains("url") || content.contains("URL") {
            ui::print_warning(&format!(
                "{} - Using getenv in URL string - will be 'None' if env var missing!",
                path.file_name().unwrap_or_default().to_string_lossy()
            ));
            issues += 1;
        }
    }

    Ok(issues)
}

fn process_python_error(stderr: &str) -> Result<usize> {
    let mut count = 0;
    
    if stderr.contains("Traceback") || stderr.contains("Error:") {
        let lines: Vec<&str> = stderr.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.contains("File \"") && line.contains(", line ") {
                ui::print_info(line.trim());
            }
            
            if line.contains("Error:") || line.contains("Exception:") {
                println!();
                ui::print_error(line.trim());
                count += 1;

                // Show fix suggestion
                println!();
                fixer::analyze_error(stderr)?;
                break;
            }
        }
    }

    Ok(count)
}

fn process_compiler_errors(output: &str) -> Result<usize> {
    let mut count = 0;

    for line in output.lines() {
        if line.contains("error:") {
            ui::print_error(line);
            count += 1;

            if count == 1 {
                println!();
                fixer::analyze_error(output)?;
            }
        }
    }

    Ok(count)
}

fn check_javascript(path: &Path) -> Result<usize> {
    let mut error_count = 0;

    let files: Vec<_> = WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| {
                    let ext = ext.to_string_lossy().to_lowercase();
                    matches!(ext.as_str(), "js" | "jsx" | "mjs")
                })
                .unwrap_or(false)
        })
        .filter(|e| !e.path().to_string_lossy().contains("node_modules"))
        .collect();

    for entry in files {
        let file_path = entry.path();
        
        let file_str = file_path.to_string_lossy().to_string();
        let file_str = file_str.strip_prefix(r"\\?\").unwrap_or(&file_str);

        ui::print_info(&format!("Checking: {}", file_str));
        
        let syntax_output = Command::new("node")
            .args(["--check", file_str])
            .output();

        if let Ok(output) = syntax_output {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error_count += process_js_error(&stderr, file_str)?;
                continue;
            }
        }
        
        let run_output = Command::new("node")
            .arg(file_str)
            .current_dir(&path)
            .output();

        if let Ok(output) = run_output {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    error_count += process_js_error(&stderr, file_str)?;
                }
            }
        }
    }

    Ok(error_count)
}

fn process_js_error(stderr: &str, file_path: &str) -> Result<usize> {
    let mut count = 0;
    
    if stderr.contains("Cannot find module") {
        let module_re = regex::Regex::new(r"Cannot find module '([^']+)'").ok();
        let module_name = module_re
            .and_then(|re| re.captures(stderr))
            .map(|cap| cap[1].to_string())
            .unwrap_or_else(|| "unknown".to_string());

        println!();
        ui::print_error(&format!("Module not found: '{}'", module_name));
        ui::print_file_location(file_path, Some(1), None);
        println!();

        ui::print_section("How to Fix");
        println!();
        println!("  Install the missing module:");
        println!();
        println!("    npm install {}", module_name);
        println!();

        count += 1;
        return Ok(count);
    }
    
    if stderr.contains("SyntaxError") {
        println!();
        ui::print_error("Syntax Error in JavaScript");
        ui::print_file_location(file_path, None, None);
        println!();
        
        for line in stderr.lines() {
            if line.contains("SyntaxError:") {
                ui::print_error(line.trim());
                break;
            }
        }

        println!();
        fixer::analyze_error(stderr)?;
        count += 1;
        return Ok(count);
    }
    
    if stderr.contains("ReferenceError") || stderr.contains("TypeError") {
        for line in stderr.lines() {
            if line.contains("Error:") {
                println!();
                ui::print_error(line.trim());
                count += 1;
                break;
            }
        }

        if count > 0 {
            ui::print_file_location(file_path, None, None);
            println!();
            fixer::analyze_error(stderr)?;
        }
    }
    
    if count == 0 && stderr.contains("Error") {
        println!();
        ui::print_error(&format!("Error in {}", file_path));
        
        for line in stderr.lines() {
            let line = line.trim();
            if line.contains("Error:") || line.contains("error:") {
                ui::print_error(line);
                count += 1;
                break;
            }
        }

        if count == 0 {
            for line in stderr.lines().take(5) {
                println!("  {}", line);
            }
            count += 1;
        }
    }

    Ok(count)
}

fn check_typescript(path: &Path) -> Result<usize> {
    let output = Command::new("npx")
        .current_dir(path)
        .args(["tsc", "--noEmit"])
        .output();

    if let Ok(output) = output {
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return process_compiler_errors(&stdout);
        }
    }

    Ok(0)
}

fn check_rust(path: &Path) -> Result<usize> {
    let cargo_toml = path.join("Cargo.toml");

    if cargo_toml.exists() {
        let output = Command::new("cargo")
            .current_dir(path)
            .args(["check", "--message-format=short"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return process_compiler_errors(&stderr);
        }
    }

    Ok(0)
}
