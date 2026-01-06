use owo_colors::OwoColorize;

const GRADIENT_START: (u8, u8, u8) = (255, 240, 181); // #FFF0B5
const GRADIENT_END: (u8, u8, u8) = (134, 69, 199); // #8645C7
const SUCCESS: (u8, u8, u8) = (134, 239, 172); // Green
const ERROR: (u8, u8, u8) = (248, 113, 113); // Red
const WARNING: (u8, u8, u8) = (251, 191, 36); // Amber
const INFO: (u8, u8, u8) = (147, 197, 253); // Blue
const DIM: (u8, u8, u8) = (148, 163, 184); // Gray

pub fn print_banner() {
    let banner = r#"
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║   ███████╗███████╗███████╗  ╔═╗╔═╗╔╦╗╔═╗                      ║
    ║   ██╔════╝██╔════╝██╔════╝  ║  ║ ║ ║║║╣                       ║
    ║   █████╗  ███████╗███████╗  ╚═╝╚═╝═╩╝╚═╝                      ║
    ║   ██╔══╝  ╚════██║╚════██║                                    ║
    ║   ███████╗███████║███████║  Smart Error Fixer                 ║
    ║   ╚══════╝╚══════╝╚══════╝  v0.2.0                            ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
"#;

    print_gradient(banner);
    println!();
}

pub fn print_gradient(text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    let total = lines.len().max(1) as f32;

    for (i, line) in lines.iter().enumerate() {
        let t = i as f32 / total;
        let r = lerp(GRADIENT_START.0, GRADIENT_END.0, t);
        let g = lerp(GRADIENT_START.1, GRADIENT_END.1, t);
        let b = lerp(GRADIENT_START.2, GRADIENT_END.2, t);
        println!("{}", line.truecolor(r, g, b));
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

pub fn print_section(title: &str) {
    println!();
    let line = "─".repeat(60);
    println!("{}", line.truecolor(DIM.0, DIM.1, DIM.2));
    println!(
        "  {}",
        title
            .truecolor(GRADIENT_END.0, GRADIENT_END.1, GRADIENT_END.2)
            .bold()
    );
    println!("{}", line.truecolor(DIM.0, DIM.1, DIM.2));
}

pub fn print_success(msg: &str) {
    println!(
        "  {} {}",
        "✓".truecolor(SUCCESS.0, SUCCESS.1, SUCCESS.2).bold(),
        msg.truecolor(SUCCESS.0, SUCCESS.1, SUCCESS.2)
    );
}

pub fn print_error(msg: &str) {
    println!(
        "  {} {}",
        "✗".truecolor(ERROR.0, ERROR.1, ERROR.2).bold(),
        msg.truecolor(ERROR.0, ERROR.1, ERROR.2)
    );
}

pub fn print_warning(msg: &str) {
    println!(
        "  {} {}",
        "⚠".truecolor(WARNING.0, WARNING.1, WARNING.2).bold(),
        msg.truecolor(WARNING.0, WARNING.1, WARNING.2)
    );
}

pub fn print_info(msg: &str) {
    println!(
        "  {} {}",
        "→".truecolor(INFO.0, INFO.1, INFO.2).bold(),
        msg.truecolor(INFO.0, INFO.1, INFO.2)
    );
}

pub fn print_hint(msg: &str) {
    println!(
        "  {} {}",
        "💡".truecolor(DIM.0, DIM.1, DIM.2),
        msg.truecolor(DIM.0, DIM.1, DIM.2)
    );
}

pub fn print_file_location(file: &str, line: Option<u32>, col: Option<u32>) {
    let location = match (line, col) {
        (Some(l), Some(c)) => format!("{}:{}:{}", file, l, c),
        (Some(l), None) => format!("{}:{}", file, l),
        _ => file.to_string(),
    };
    println!(
        "  {} {}",
        "📄".truecolor(DIM.0, DIM.1, DIM.2),
        location.truecolor(INFO.0, INFO.1, INFO.2)
    );
}

pub fn print_code_line(line_num: u32, code: &str, is_error: bool) {
    let num_str = format!("{:>4} │ ", line_num);
    if is_error {
        println!(
            "{}{}",
            num_str.truecolor(ERROR.0, ERROR.1, ERROR.2),
            code.truecolor(ERROR.0, ERROR.1, ERROR.2)
        );
    } else {
        println!("{}{}", num_str.truecolor(DIM.0, DIM.1, DIM.2), code);
    }
}

pub fn print_diff(before: &str, after: &str) {
    print_section("Suggested Fix");
    println!();

    for line in before.lines() {
        println!(
            "  {} {}",
            "-".truecolor(ERROR.0, ERROR.1, ERROR.2).bold(),
            line.truecolor(ERROR.0, ERROR.1, ERROR.2)
        );
    }

    println!();

    for line in after.lines() {
        println!(
            "  {} {}",
            "+".truecolor(SUCCESS.0, SUCCESS.1, SUCCESS.2).bold(),
            line.truecolor(SUCCESS.0, SUCCESS.1, SUCCESS.2)
        );
    }

    println!();
}

pub fn print_fix_instruction(instruction: &str) {
    print_section("How to Fix");
    println!();
    for line in instruction.lines() {
        println!("  {}", line.truecolor(255, 255, 255));
    }
    println!();
}

pub fn print_supported_patterns() {
    print_section("Supported Languages & Patterns");
    println!();

    println!(
        "  {}",
        "C++ (g++/clang++)".truecolor(INFO.0, INFO.1, INFO.2).bold()
    );
    println!("    • Missing #include headers");
    println!("    • Undeclared identifiers");
    println!("    • Missing semicolons");
    println!("    • Type mismatches");
    println!();

    println!("  {}", "Python".truecolor(INFO.0, INFO.1, INFO.2).bold());
    println!("    • SyntaxError (missing colons, brackets)");
    println!("    • IndentationError");
    println!("    • NameError (undefined variables)");
    println!("    • ImportError");
    println!();

    println!(
        "  {}",
        "JavaScript/TypeScript"
            .truecolor(INFO.0, INFO.1, INFO.2)
            .bold()
    );
    println!("    • SyntaxError (unexpected tokens)");
    println!("    • ReferenceError");
    println!("    • TypeError");
    println!("    • Module not found");
    println!();

    println!("  {}", "Rust".truecolor(INFO.0, INFO.1, INFO.2).bold());
    println!("    • Missing use statements");
    println!("    • Borrow checker errors");
    println!("    • Type mismatches");
    println!();

    print_hint("More patterns coming soon!");
    println!();
}

pub fn print_no_errors() {
    println!();
    println!(
        "  {} {}",
        "✓".truecolor(SUCCESS.0, SUCCESS.1, SUCCESS.2).bold(),
        "No errors found!"
            .truecolor(SUCCESS.0, SUCCESS.1, SUCCESS.2)
            .bold()
    );
    println!();
}

pub fn print_errors_found(count: usize) {
    println!();
    println!(
        "  {} {} error{} found",
        "●".truecolor(ERROR.0, ERROR.1, ERROR.2).bold(),
        count
            .to_string()
            .truecolor(ERROR.0, ERROR.1, ERROR.2)
            .bold(),
        if count == 1 { "" } else { "s" }
    );
}
