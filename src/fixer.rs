use crate::parser::{parse_error, ErrorType, Language, ParsedError};
use crate::ui;
use anyhow::Result;

pub fn analyze_error(error_text: &str) -> Result<()> {
    ui::print_section("Analyzing Error");

    if let Some(error) = parse_error(error_text) {
        show_parsed_error(&error);
        show_fix_for_error(&error);
    } else {
        ui::print_warning("Could not fully parse error format");
        ui::print_info("Attempting pattern matching...");
        println!();

        if let Some(fix) = try_common_patterns(error_text) {
            ui::print_fix_instruction(&fix);
        } else {
            ui::print_error("Unknown error pattern");
            ui::print_hint("Try 'ess list' to see supported error types");
        }
    }

    Ok(())
}

fn show_parsed_error(error: &ParsedError) {
    println!();
    ui::print_info(&format!("Language: {}", error.language));
    ui::print_file_location(&error.file, error.line, error.column);
    println!();
    ui::print_error(&error.message);
}

fn show_fix_for_error(error: &ParsedError) {
    match &error.error_type {
        ErrorType::MissingInclude(header) => {
            fix_missing_include(header, &error.language);
        }
        ErrorType::MissingSemicolon => {
            fix_missing_semicolon(&error.language);
        }
        ErrorType::UndeclaredVariable(var) => {
            fix_undeclared_variable(var, &error.language);
        }
        ErrorType::SyntaxError(details) => {
            fix_syntax_error(details, &error.language);
        }
        ErrorType::IndentationError => {
            fix_indentation_error();
        }
        ErrorType::ImportError(module) => {
            fix_import_error(module, &error.language);
        }
        ErrorType::ModuleNotFound(module) => {
            fix_module_not_found(module, &error.language);
        }
        ErrorType::TypeError(details) => {
            fix_type_error(details, &error.language);
        }
        ErrorType::BorrowError(details) => {
            fix_borrow_error(details);
        }
        ErrorType::KeyError(key) => {
            fix_key_error(key);
        }
        ErrorType::AttributeError(details) => {
            fix_attribute_error(details);
        }
        ErrorType::ValueError(details) => {
            fix_value_error(details);
        }
        ErrorType::MissingEnvVar(details) => {
            fix_missing_env_var(details);
        }
        ErrorType::RequestsError(details) => {
            fix_requests_error(details);
        }
        ErrorType::Unknown(msg) => {
            ui::print_warning(&format!("No automatic fix for: {}", msg));
            ui::print_hint("Check the error message and fix manually");
        }
    }
}

fn fix_missing_include(header: &str, lang: &Language) {
    if lang == &Language::Cpp {
        let before = "// Your current code";
        let after = format!("#include <{}>\n// Your code", header);

        ui::print_diff(before, &after);
        ui::print_fix_instruction(&format!(
            "Add this line at the top of your file:\n\n  #include <{}>",
            header
        ));
    }
}

fn fix_missing_semicolon(lang: &Language) {
    match lang {
        Language::Cpp | Language::JavaScript | Language::TypeScript => {
            ui::print_diff("statement  // missing semicolon", "statement;");
            ui::print_fix_instruction(
                "Add a semicolon at the end of the line indicated in the error.\n\n\
                Look for the line number in the error message and add ';' at the end.",
            );
        }
        _ => {}
    }
}

fn fix_undeclared_variable(var: &str, lang: &Language) {
    ui::print_section("Possible Causes");
    println!();

    ui::print_info(&format!("Variable '{}' is not defined", var));
    println!();

    match lang {
        Language::Cpp => {
            println!("  1. Typo in variable name");
            println!("  2. Variable declared in different scope");
            println!("  3. Missing #include for std:: types");
            println!();

            if is_std_type(var) {
                ui::print_diff(
                    &format!("std::{}", var),
                    &format!("#include <{}>\nstd::{}", var.to_lowercase(), var),
                );
            } else {
                ui::print_fix_instruction(&format!(
                    "Options:\n\n\
                    1. Check spelling of '{}'\n\
                    2. Declare the variable before using it:\n   int {} = 0;\n\
                    3. Check if it's defined in a different scope",
                    var, var
                ));
            }
        }
        Language::Python => {
            ui::print_fix_instruction(&format!(
                "Options:\n\n\
                1. Check spelling of '{}'\n\
                2. Define the variable before using it:\n   {} = None\n\
                3. Make sure the variable is in scope",
                var, var
            ));
        }
        Language::JavaScript | Language::TypeScript => {
            ui::print_fix_instruction(&format!(
                "Options:\n\n\
                1. Check spelling of '{}'\n\
                2. Declare the variable:\n   const {} = ...;\n\
                3. Import if it's from another module:\n   import {{ {} }} from './module';",
                var, var, var
            ));
        }
        Language::Rust => {
            ui::print_fix_instruction(&format!(
                "Options:\n\n\
                1. Check spelling of '{}'\n\
                2. Add a 'use' statement if it's from another module:\n   use crate::{};\n\
                3. Declare the variable:\n   let {} = ...;",
                var, var, var
            ));
        }
        _ => {}
    }
}

fn fix_syntax_error(details: &str, _lang: &Language) {
    ui::print_section("Syntax Error");
    println!();

    let details_lower = details.to_lowercase();

    if details_lower.contains("unexpected token") {
        ui::print_fix_instruction(
            "Check for:\n\n\
            1. Missing or extra brackets: { } [ ] ( )\n\
            2. Missing commas in arrays or objects\n\
            3. Unclosed strings\n\
            4. Missing operators",
        );
    } else if details_lower.contains("was never closed") || details_lower.contains("unterminated") {
        ui::print_fix_instruction(
            "You have an unclosed bracket or string.\n\n\
            Check for matching pairs:\n\
            • ( must have )\n\
            • { must have }\n\
            • [ must have ]\n\
            • \" must have \"\n\
            • ' must have '",
        );
    } else if details_lower.contains("expected") {
        ui::print_fix_instruction(&format!(
            "The parser expected something that wasn't there.\n\n\
            Error: {}\n\n\
            Check the line number in the error for missing syntax.",
            details
        ));
    } else {
        ui::print_fix_instruction(&format!(
            "Syntax error: {}\n\n\
            Check the line indicated in the error for typos or missing syntax.",
            details
        ));
    }
}

fn fix_indentation_error() {
    ui::print_diff(
        "def example():\n  line1  # 2 spaces\n    line2  # 4 spaces (inconsistent!)",
        "def example():\n    line1  # 4 spaces\n    line2  # 4 spaces (consistent)",
    );
    ui::print_fix_instruction(
        "Python requires consistent indentation.\n\n\
        Fix:\n\
        1. Use either spaces OR tabs, not both\n\
        2. Use 4 spaces per indentation level (recommended)\n\
        3. Make sure all lines in a block have the same indentation\n\n\
        Tip: Configure your editor to convert tabs to spaces.",
    );
}

fn fix_import_error(module: &str, lang: &Language) {
    match lang {
        Language::Python => {
            ui::print_fix_instruction(&format!(
                "Module '{}' not found.\n\n\
                Options:\n\n\
                1. Install the module:\n   pip install {}\n\n\
                2. Check if it's a local module - verify the file exists\n\n\
                3. Check your PYTHONPATH if it's a custom module",
                module, module
            ));
        }
        _ => {
            ui::print_fix_instruction(&format!(
                "Module '{}' not found.\n\n\
                Check that the module is installed and the path is correct.",
                module
            ));
        }
    }
}

fn fix_module_not_found(module: &str, lang: &Language) {
    match lang {
        Language::JavaScript | Language::TypeScript => {
            ui::print_fix_instruction(&format!(
                "Cannot find module '{}'\n\n\
                Options:\n\n\
                1. Install the package:\n   npm install {}\n\n\
                2. If it's a local file, check the path:\n   import x from './{}'\n\n\
                3. Check tsconfig.json paths if using TypeScript",
                module, module, module
            ));
        }
        _ => {
            ui::print_fix_instruction(&format!(
                "Module '{}' not found. Check installation and import path.",
                module
            ));
        }
    }
}

fn fix_type_error(details: &str, lang: &Language) {
    ui::print_section("Type Error");
    println!();

    ui::print_error(details);
    println!();

    match lang {
        Language::TypeScript => {
            ui::print_fix_instruction(
                "Type mismatch detected.\n\n\
                Options:\n\n\
                1. Check the expected type vs what you're passing\n\
                2. Add type assertion: value as ExpectedType\n\
                3. Fix the source of the wrong type\n\
                4. Update the type definition if it's incorrect",
            );
        }
        Language::Python => {
            ui::print_fix_instruction(
                "Operation not supported for this type.\n\n\
                Check what type your variable actually is:\n  print(type(your_variable))\n\n\
                Then ensure the operation is valid for that type.",
            );
        }
        _ => {
            ui::print_fix_instruction(
                "Type mismatch. Check that your variables have the expected types.",
            );
        }
    }
}

fn fix_borrow_error(details: &str) {
    ui::print_section("Borrow Checker Error");
    println!();

    ui::print_error(details);
    println!();

    ui::print_fix_instruction(
        "Rust's borrow checker prevents data races.\n\n\
        Common fixes:\n\n\
        1. Clone the data if ownership isn't needed:\n   let copy = data.clone();\n\n\
        2. Use references instead of moving:\n   fn process(data: &MyType) { ... }\n\n\
        3. Limit the scope of borrows:\n   {\n       let r = &mut data;\n       // use r\n   } // r dropped here\n\n\
        4. Use Rc/Arc for shared ownership:\n   use std::rc::Rc;",
    );
}

fn try_common_patterns(error_text: &str) -> Option<String> {
    let lower = error_text.to_lowercase();

    if lower.contains("expected ';'") || lower.contains("missing semicolon") {
        return Some("Add a semicolon (;) at the end of the line.".to_string());
    }

    if lower.contains("is not a member of") || lower.contains("was not declared") {
        return Some(
            "You're using something that hasn't been imported/included.\n\
            Add the appropriate #include or import statement at the top of your file."
                .to_string(),
        );
    }

    if lower.contains("is not defined") || lower.contains("undeclared") {
        return Some(
            "Variable is not defined.\n\
            Either declare it before using, or check for typos in the name."
                .to_string(),
        );
    }

    if lower.contains("unexpected token") || lower.contains("was never closed") {
        return Some(
            "Syntax error - check for:\n\
            • Missing or extra brackets { } [ ] ( )\n\
            • Unclosed strings\n\
            • Missing semicolons or commas"
                .to_string(),
        );
    }

    None
}

fn is_std_type(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "vector"
            | "string"
            | "map"
            | "set"
            | "list"
            | "deque"
            | "array"
            | "unique_ptr"
            | "shared_ptr"
            | "optional"
            | "variant"
    )
}

fn fix_key_error(key: &str) {
    ui::print_section("KeyError - Missing Dictionary Key");
    println!();

    ui::print_diff(
        &format!("data[\"{}\"]  # raises KeyError if missing", key),
        &format!(
            "data.get(\"{}\", default_value)  # returns default if missing",
            key
        ),
    );

    ui::print_fix_instruction(&format!(
        "The key '{}' doesn't exist in the dictionary.\n\n\
        Options:\n\n\
        1. Use .get() with a default value:\n\
           value = data.get(\"{}\", None)\n\n\
        2. Check if key exists first:\n\
           if \"{}\" in data:\n\
               value = data[\"{}\"]\n\n\
        3. Use try/except:\n\
           try:\n\
               value = data[\"{}\"]\n\
           except KeyError:\n\
               value = default",
        key, key, key, key, key
    ));
}

fn fix_attribute_error(details: &str) {
    ui::print_section("AttributeError");
    println!();

    if details.contains("'NoneType'") {
        ui::print_diff(
            "result.method()  # result is None!",
            "if result is not None:\n    result.method()",
        );

        ui::print_fix_instruction(
            "You're calling a method on a None value.\n\n\
            The variable is None when you expected an object.\n\n\
            Fix:\n\n\
            1. Check for None before using:\n\
               if result is not None:\n\
                   result.method()\n\n\
            2. Use a default value:\n\
               result = get_result() or default_value\n\n\
            3. Find why the value is None and fix the source",
        );
    } else {
        ui::print_fix_instruction(&format!(
            "AttributeError: {}\n\n\
            The object doesn't have the attribute/method you're trying to use.\n\n\
            Check:\n\
            1. Spelling of the attribute name\n\
            2. The type of the object (use type(obj))\n\
            3. If the object is None unexpectedly",
            details
        ));
    }
}

fn fix_value_error(details: &str) {
    ui::print_section("ValueError");
    println!();

    if details.contains("fromisoformat") || details.contains("time data") {
        ui::print_diff(
            "datetime.fromisoformat(date_string)  # fails if invalid",
            "try:\n    dt = datetime.fromisoformat(date_string)\nexcept (ValueError, TypeError):\n    dt = None",
        );

        ui::print_fix_instruction(
            "The datetime string is invalid or None.\n\n\
            Fix:\n\n\
            1. Validate before parsing:\n\
               if date_string:\n\
                   dt = datetime.fromisoformat(date_string)\n\n\
            2. Use try/except:\n\
               try:\n\
                   dt = datetime.fromisoformat(date_string)\n\
               except (ValueError, TypeError):\n\
                   dt = datetime.now()  # or None",
        );
    } else {
        ui::print_fix_instruction(&format!(
            "ValueError: {}\n\n\
            The value has the right type but invalid content.\n\n\
            Validate the data before using it.",
            details
        ));
    }
}

fn fix_missing_env_var(_details: &str) {
    ui::print_section("Missing Environment Variable");
    println!();

    ui::print_error("Environment variable is not set - value is None!");
    println!();

    ui::print_diff(
        "API_URL = os.getenv(\"API_URL\")  # Returns None if not set!\nurl = f\"{API_URL}/endpoint\"  # Becomes 'None/endpoint'",
        "API_URL = os.getenv(\"API_URL\")\nif not API_URL:\n    raise ValueError(\"API_URL environment variable is required\")\nurl = f\"{API_URL}/endpoint\"",
    );

    ui::print_fix_instruction(
        "os.getenv() returns None when the variable isn't set.\n\n\
        Fix:\n\n\
        1. Set the environment variable:\n\
           - Create/edit .env file: API_URL=https://api.example.com\n\
           - Or set in terminal: export API_URL=https://api.example.com\n\n\
        2. Add validation in your code:\n\
           API_URL = os.getenv(\"API_URL\")\n\
           if not API_URL:\n\
               raise ValueError(\"API_URL is required\")\n\n\
        3. Use a default value:\n\
           API_URL = os.getenv(\"API_URL\", \"https://default-api.com\")",
    );
}

fn fix_requests_error(details: &str) {
    ui::print_section("Requests Library Error");
    println!();

    ui::print_error(details);
    println!();

    if details.contains("ConnectionError") || details.contains("connect") {
        ui::print_fix_instruction(
            "Could not connect to the server.\n\n\
            Check:\n\
            1. Is the URL correct?\n\
            2. Is the server running?\n\
            3. Is your internet connection working?\n\
            4. Is there a firewall blocking the request?",
        );
    } else if details.contains("Timeout") {
        ui::print_fix_instruction(
            "Request timed out.\n\n\
            Fix:\n\
            1. Increase the timeout:\n\
               requests.get(url, timeout=30)\n\n\
            2. Check if the server is slow/overloaded\n\
            3. Add retry logic:\n\
               from requests.adapters import HTTPAdapter\n\
               from urllib3.util.retry import Retry",
        );
    } else {
        ui::print_fix_instruction(
            "Add proper error handling:\n\n\
            try:\n\
                response = requests.get(url, timeout=10)\n\
                response.raise_for_status()\n\
            except requests.exceptions.RequestException as e:\n\
                print(f\"Request failed: {e}\")",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== try_common_patterns Tests ====================

    #[test]
    fn test_pattern_missing_semicolon() {
        let result = try_common_patterns("expected ';' before return");
        assert!(result.is_some());
        assert!(result.unwrap().contains("semicolon"));
    }

    #[test]
    fn test_pattern_missing_semicolon_variant() {
        let result = try_common_patterns("missing semicolon at end of line");
        assert!(result.is_some());
        assert!(result.unwrap().contains("semicolon"));
    }

    #[test]
    fn test_pattern_not_a_member() {
        let result = try_common_patterns("'vector' is not a member of 'std'");
        assert!(result.is_some());
        let msg = result.unwrap();
        assert!(msg.contains("import") || msg.contains("include"));
    }

    #[test]
    fn test_pattern_was_not_declared() {
        let result = try_common_patterns("'myVar' was not declared in this scope");
        assert!(result.is_some());
        let msg = result.unwrap();
        assert!(msg.contains("import") || msg.contains("include"));
    }

    #[test]
    fn test_pattern_is_not_defined() {
        let result = try_common_patterns("ReferenceError: x is not defined");
        assert!(result.is_some());
        let msg = result.unwrap();
        assert!(msg.contains("define") || msg.contains("declare"));
    }

    #[test]
    fn test_pattern_undeclared() {
        let result = try_common_patterns("use of undeclared identifier 'foo'");
        assert!(result.is_some());
    }

    #[test]
    fn test_pattern_unexpected_token() {
        let result = try_common_patterns("SyntaxError: unexpected token '}'");
        assert!(result.is_some());
        let msg = result.unwrap();
        assert!(msg.contains("bracket") || msg.contains("Syntax"));
    }

    #[test]
    fn test_pattern_was_never_closed() {
        let result = try_common_patterns("string literal was never closed");
        assert!(result.is_some());
    }

    #[test]
    fn test_pattern_no_match() {
        let result = try_common_patterns("some random unrecognized error");
        assert!(result.is_none());
    }

    #[test]
    fn test_pattern_empty_input() {
        let result = try_common_patterns("");
        assert!(result.is_none());
    }

    // ==================== is_std_type Tests ====================

    #[test]
    fn test_is_std_type_vector() {
        assert!(is_std_type("vector"));
        assert!(is_std_type("Vector"));
        assert!(is_std_type("VECTOR"));
    }

    #[test]
    fn test_is_std_type_string() {
        assert!(is_std_type("string"));
        assert!(is_std_type("String"));
    }

    #[test]
    fn test_is_std_type_map() {
        assert!(is_std_type("map"));
        assert!(is_std_type("Map"));
    }

    #[test]
    fn test_is_std_type_set() {
        assert!(is_std_type("set"));
        assert!(is_std_type("Set"));
    }

    #[test]
    fn test_is_std_type_smart_pointers() {
        assert!(is_std_type("unique_ptr"));
        assert!(is_std_type("shared_ptr"));
    }

    #[test]
    fn test_is_std_type_containers() {
        assert!(is_std_type("list"));
        assert!(is_std_type("deque"));
        assert!(is_std_type("array"));
    }

    #[test]
    fn test_is_std_type_modern_cpp() {
        assert!(is_std_type("optional"));
        assert!(is_std_type("variant"));
    }

    #[test]
    fn test_is_std_type_not_std() {
        assert!(!is_std_type("MyClass"));
        assert!(!is_std_type("foo"));
        assert!(!is_std_type("random_name"));
    }

    // ==================== ErrorType Handling Tests ====================

    #[test]
    fn test_error_type_variants_exist() {
        // Verify all error types can be matched
        let types = vec![
            ErrorType::MissingInclude("test".to_string()),
            ErrorType::MissingSemicolon,
            ErrorType::UndeclaredVariable("var".to_string()),
            ErrorType::SyntaxError("details".to_string()),
            ErrorType::IndentationError,
            ErrorType::ImportError("module".to_string()),
            ErrorType::TypeError("info".to_string()),
            ErrorType::ModuleNotFound("mod".to_string()),
            ErrorType::BorrowError("borrow".to_string()),
            ErrorType::KeyError("key".to_string()),
            ErrorType::AttributeError("attr".to_string()),
            ErrorType::ValueError("val".to_string()),
            ErrorType::MissingEnvVar("VAR".to_string()),
            ErrorType::RequestsError("req".to_string()),
            ErrorType::Unknown("unknown".to_string()),
        ];

        assert_eq!(types.len(), 15);
    }

    // ==================== Integration-style Tests ====================

    #[test]
    fn test_analyze_error_does_not_panic_on_valid_input() {
        // These should not panic
        let test_cases = vec![
            r#"File "test.py", line 5
SyntaxError: invalid syntax"#,
            "main.cpp:10:5: error: expected ';' before 'return'",
            r#"error[E0425]: cannot find value `x`
 --> src/main.rs:5:10"#,
        ];

        for case in test_cases {
            let result = analyze_error(case);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_analyze_error_handles_unknown_format() {
        let result = analyze_error("completely random text");
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_error_handles_empty_input() {
        let result = analyze_error("");
        assert!(result.is_ok());
    }
}
