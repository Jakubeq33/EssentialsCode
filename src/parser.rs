use regex::Regex;

#[derive(Debug, Clone)]
pub struct ParsedError {
    pub file: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub message: String,
    pub error_type: ErrorType,
    pub language: Language,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    MissingInclude(String),
    MissingSemicolon,
    UndeclaredVariable(String),
    SyntaxError(String),
    IndentationError,
    ImportError(String),
    TypeError(String),
    ModuleNotFound(String),
    BorrowError(String),
    KeyError(String),
    AttributeError(String),
    ValueError(String),
    MissingEnvVar(String),
    RequestsError(String),
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Cpp,
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Unknown,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Cpp => write!(f, "C++"),
            Language::Python => write!(f, "Python"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::Rust => write!(f, "Rust"),
            Language::Unknown => write!(f, "Unknown"),
        }
    }
}

pub fn parse_error(input: &str) -> Option<ParsedError> {
    if let Some(err) = parse_cpp_error(input) {
        return Some(err);
    }
    if let Some(err) = parse_python_error(input) {
        return Some(err);
    }
    if let Some(err) = parse_js_error(input) {
        return Some(err);
    }
    if let Some(err) = parse_rust_error(input) {
        return Some(err);
    }

    None
}

fn parse_cpp_error(input: &str) -> Option<ParsedError> {
    let re = Regex::new(r"([^\s:]+\.(cpp|cc|cxx|c|h|hpp)):(\d+):(\d+): error: (.+)").ok()?;

    if let Some(cap) = re.captures(input) {
        let file = cap[1].to_string();
        let line: u32 = cap[3].parse().ok()?;
        let col: u32 = cap[4].parse().ok()?;
        let message = cap[5].to_string();

        let error_type = detect_cpp_error_type(&message, input);

        return Some(ParsedError {
            file,
            line: Some(line),
            column: Some(col),
            message,
            error_type,
            language: Language::Cpp,
        });
    }

    None
}

fn detect_cpp_error_type(message: &str, full: &str) -> ErrorType {
    let msg = message.to_lowercase();

    if msg.contains("is not a member of 'std'") || msg.contains("was not declared") {
        let include_re = Regex::new(r"#include <([^>]+)>").ok();
        if let Some(re) = include_re {
            if let Some(cap) = re.captures(full) {
                return ErrorType::MissingInclude(cap[1].to_string());
            }
        }

        if msg.contains("vector") {
            return ErrorType::MissingInclude("vector".to_string());
        }
        if msg.contains("string") {
            return ErrorType::MissingInclude("string".to_string());
        }
        if msg.contains("cout") || msg.contains("cin") {
            return ErrorType::MissingInclude("iostream".to_string());
        }
        if msg.contains("map") {
            return ErrorType::MissingInclude("map".to_string());
        }
        if msg.contains("set") {
            return ErrorType::MissingInclude("set".to_string());
        }
    }

    if msg.contains("expected ';'") || msg.contains("expected ';' before") {
        return ErrorType::MissingSemicolon;
    }

    let undecl_re =
        Regex::new(r"'([^']+)' was not declared|use of undeclared identifier '([^']+)'").ok();
    if let Some(re) = undecl_re {
        if let Some(cap) = re.captures(&msg) {
            let var = cap.get(1).or(cap.get(2)).map(|m| m.as_str().to_string());
            if let Some(v) = var {
                return ErrorType::UndeclaredVariable(v);
            }
        }
    }

    ErrorType::Unknown(message.to_string())
}

fn parse_python_error(input: &str) -> Option<ParsedError> {
    let file_re = Regex::new(r#"File "([^"]+\.py)", line (\d+)"#).ok()?;
    let error_re = Regex::new(r"(SyntaxError|IndentationError|NameError|ImportError|TypeError|ModuleNotFoundError|KeyError|AttributeError|ValueError|requests\.exceptions\.\w+): (.+)").ok()?;

    let requests_re = Regex::new(r"requests\.exceptions\.(\w+): (.+)").ok()?;

    let file_cap = file_re.captures(input);
    let error_cap = error_re.captures(input);

    if let Some(req_cap) = requests_re.captures(input) {
        let error_name = req_cap[1].to_string();
        let details = req_cap[2].to_string();

        let error_type = if error_name == "MissingSchema" || details.contains("None") {
            ErrorType::MissingEnvVar(details.clone())
        } else {
            ErrorType::RequestsError(format!("{}: {}", error_name, details))
        };

        let file = file_cap
            .as_ref()
            .map(|c| c[1].to_string())
            .unwrap_or_else(|| "unknown.py".to_string());
        let line = file_cap.as_ref().and_then(|c| c[2].parse().ok());

        return Some(ParsedError {
            file,
            line,
            column: None,
            message: format!("requests.exceptions.{}: {}", error_name, details),
            error_type,
            language: Language::Python,
        });
    }

    if let (Some(fc), Some(ec)) = (file_cap, error_cap) {
        let file = fc[1].to_string();
        let line: u32 = fc[2].parse().ok()?;
        let error_name = &ec[1];
        let details = ec[2].to_string();

        let error_type = match error_name {
            "SyntaxError" => ErrorType::SyntaxError(details.clone()),
            "IndentationError" => ErrorType::IndentationError,
            "NameError" => {
                let var_re = Regex::new(r"name '([^']+)' is not defined").ok();
                if let Some(re) = var_re {
                    if let Some(cap) = re.captures(&details) {
                        ErrorType::UndeclaredVariable(cap[1].to_string())
                    } else {
                        ErrorType::Unknown(details.clone())
                    }
                } else {
                    ErrorType::Unknown(details.clone())
                }
            }
            "ImportError" | "ModuleNotFoundError" => {
                let mod_re = Regex::new(r"No module named '([^']+)'").ok();
                if let Some(re) = mod_re {
                    if let Some(cap) = re.captures(&details) {
                        ErrorType::ImportError(cap[1].to_string())
                    } else {
                        ErrorType::ImportError(details.clone())
                    }
                } else {
                    ErrorType::ImportError(details.clone())
                }
            }
            "TypeError" => ErrorType::TypeError(details.clone()),
            "KeyError" => ErrorType::KeyError(details.clone()),
            "AttributeError" => ErrorType::AttributeError(details.clone()),
            "ValueError" => ErrorType::ValueError(details.clone()),
            _ => ErrorType::Unknown(details.clone()),
        };

        return Some(ParsedError {
            file,
            line: Some(line),
            column: None,
            message: format!("{}: {}", error_name, details),
            error_type,
            language: Language::Python,
        });
    }

    None
}

fn parse_js_error(input: &str) -> Option<ParsedError> {
    let file_re = Regex::new(r"([^\s:]+\.(js|ts|jsx|tsx|mjs)):(\d+)(?::(\d+))?").ok()?;
    let error_re = Regex::new(r"(SyntaxError|TypeError|ReferenceError): (.+)").ok()?;

    let ts_re = Regex::new(r"([^\s(]+\.(ts|tsx))\((\d+),(\d+)\): error (TS\d+): (.+)").ok()?;

    if let Some(cap) = ts_re.captures(input) {
        let file = cap[1].to_string();
        let line: u32 = cap[3].parse().ok()?;
        let col: u32 = cap[4].parse().ok()?;
        let code = &cap[5];
        let message = cap[6].to_string();

        let error_type = match code {
            "TS2304" | "TS2552" => {
                let var_re = Regex::new(r"Cannot find name '([^']+)'").ok();
                if let Some(re) = var_re {
                    if let Some(c) = re.captures(&message) {
                        ErrorType::UndeclaredVariable(c[1].to_string())
                    } else {
                        ErrorType::Unknown(message.clone())
                    }
                } else {
                    ErrorType::Unknown(message.clone())
                }
            }
            "TS2307" => ErrorType::ModuleNotFound(message.clone()),
            _ => ErrorType::Unknown(message.clone()),
        };

        return Some(ParsedError {
            file,
            line: Some(line),
            column: Some(col),
            message: format!("{}: {}", code, message),
            error_type,
            language: Language::TypeScript,
        });
    }

    if let Some(file_cap) = file_re.captures(input) {
        if let Some(error_cap) = error_re.captures(input) {
            let file = file_cap[1].to_string();
            let ext = &file_cap[2];
            let line: u32 = file_cap[3].parse().ok()?;
            let col: Option<u32> = file_cap.get(4).and_then(|m| m.as_str().parse().ok());

            let error_name = &error_cap[1];
            let details = error_cap[2].to_string();

            let language = if ext == "ts" || ext == "tsx" {
                Language::TypeScript
            } else {
                Language::JavaScript
            };

            let error_type = match error_name {
                "SyntaxError" => ErrorType::SyntaxError(details.clone()),
                "ReferenceError" => {
                    let var_re = Regex::new(r"(\w+) is not defined").ok();
                    if let Some(re) = var_re {
                        if let Some(cap) = re.captures(&details) {
                            ErrorType::UndeclaredVariable(cap[1].to_string())
                        } else {
                            ErrorType::Unknown(details.clone())
                        }
                    } else {
                        ErrorType::Unknown(details.clone())
                    }
                }
                "TypeError" => ErrorType::TypeError(details.clone()),
                _ => ErrorType::Unknown(details.clone()),
            };

            return Some(ParsedError {
                file,
                line: Some(line),
                column: col,
                message: format!("{}: {}", error_name, details),
                error_type,
                language,
            });
        }
    }

    None
}

fn parse_rust_error(input: &str) -> Option<ParsedError> {
    let error_re = Regex::new(r"error\[E\d+\]: (.+)").ok()?;
    let loc_re = Regex::new(r"--> ([^:]+):(\d+):(\d+)").ok()?;

    let error_cap = error_re.captures(input);
    let loc_cap = loc_re.captures(input);

    if let (Some(ec), Some(lc)) = (error_cap, loc_cap) {
        let message = ec[1].to_string();
        let file = lc[1].to_string();
        let line: u32 = lc[2].parse().ok()?;
        let col: u32 = lc[3].parse().ok()?;

        let error_type = if message.contains("cannot find") {
            let var_re = Regex::new(r"cannot find (?:value|type) `([^`]+)`").ok();
            if let Some(re) = var_re {
                if let Some(cap) = re.captures(&message) {
                    ErrorType::UndeclaredVariable(cap[1].to_string())
                } else {
                    ErrorType::Unknown(message.clone())
                }
            } else {
                ErrorType::Unknown(message.clone())
            }
        } else if message.contains("borrow") {
            ErrorType::BorrowError(message.clone())
        } else {
            ErrorType::Unknown(message.clone())
        };

        return Some(ParsedError {
            file,
            line: Some(line),
            column: Some(col),
            message,
            error_type,
            language: Language::Rust,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== C++ Parser Tests ====================

    #[test]
    fn test_parse_cpp_missing_include() {
        let error = "main.cpp:5:10: error: 'vector' is not a member of 'std'";
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.language, Language::Cpp);
        assert_eq!(parsed.file, "main.cpp");
        assert_eq!(parsed.line, Some(5));
        assert_eq!(parsed.column, Some(10));
        assert!(matches!(parsed.error_type, ErrorType::MissingInclude(_)));
    }

    #[test]
    fn test_parse_cpp_missing_semicolon() {
        let error = "test.cpp:10:5: error: expected ';' before 'return'";
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.error_type, ErrorType::MissingSemicolon);
    }

    #[test]
    fn test_parse_cpp_undeclared_variable() {
        let error = "main.cpp:8:12: error: 'myVar' was not declared in this scope";
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(matches!(parsed.error_type, ErrorType::UndeclaredVariable(ref v) if v == "myvar"));
    }

    // ==================== Python Parser Tests ====================

    #[test]
    fn test_parse_python_syntax_error() {
        let error = r#"File "test.py", line 5
    def foo(
        ^
SyntaxError: unexpected EOF while parsing"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.language, Language::Python);
        assert_eq!(parsed.file, "test.py");
        assert_eq!(parsed.line, Some(5));
        assert!(matches!(parsed.error_type, ErrorType::SyntaxError(_)));
    }

    #[test]
    fn test_parse_python_indentation_error() {
        let error = r#"File "script.py", line 10
    print("hello")
    ^
IndentationError: unexpected indent"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.error_type, ErrorType::IndentationError);
    }

    #[test]
    fn test_parse_python_name_error() {
        let error = r#"File "app.py", line 15
NameError: name 'undefined_var' is not defined"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(
            matches!(parsed.error_type, ErrorType::UndeclaredVariable(ref v) if v == "undefined_var")
        );
    }

    #[test]
    fn test_parse_python_import_error() {
        let error = r#"File "main.py", line 1
ImportError: No module named 'nonexistent_module'"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(
            matches!(parsed.error_type, ErrorType::ImportError(ref m) if m == "nonexistent_module")
        );
    }

    #[test]
    fn test_parse_python_key_error() {
        let error = r#"File "data.py", line 20
KeyError: 'missing_key'"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(matches!(parsed.error_type, ErrorType::KeyError(_)));
    }

    #[test]
    fn test_parse_python_type_error() {
        let error = r#"File "calc.py", line 8
TypeError: unsupported operand type(s) for +: 'int' and 'str'"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(matches!(parsed.error_type, ErrorType::TypeError(_)));
    }

    #[test]
    fn test_parse_python_attribute_error() {
        let error = r#"File "obj.py", line 12
AttributeError: 'NoneType' object has no attribute 'split'"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(matches!(parsed.error_type, ErrorType::AttributeError(_)));
    }

    #[test]
    fn test_parse_python_value_error() {
        let error = r#"File "parse.py", line 5
ValueError: invalid literal for int() with base 10: 'abc'"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(matches!(parsed.error_type, ErrorType::ValueError(_)));
    }

    // ==================== JavaScript Parser Tests ====================

    #[test]
    fn test_parse_js_syntax_error() {
        let error = "app.js:15:20\nSyntaxError: Unexpected token '}'";
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.language, Language::JavaScript);
        assert_eq!(parsed.file, "app.js");
        assert!(matches!(parsed.error_type, ErrorType::SyntaxError(_)));
    }

    #[test]
    fn test_parse_js_reference_error() {
        let error = "index.js:8:5\nReferenceError: myFunction is not defined";
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(
            matches!(parsed.error_type, ErrorType::UndeclaredVariable(ref v) if v == "myFunction")
        );
    }

    #[test]
    fn test_parse_js_type_error() {
        let error = "utils.js:22:10\nTypeError: Cannot read property 'length' of undefined";
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(matches!(parsed.error_type, ErrorType::TypeError(_)));
    }

    // ==================== TypeScript Parser Tests ====================

    #[test]
    fn test_parse_typescript_error() {
        let error = "src/app.ts(10,15): error TS2304: Cannot find name 'unknownType'";
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.language, Language::TypeScript);
        assert_eq!(parsed.file, "src/app.ts");
        assert_eq!(parsed.line, Some(10));
        assert_eq!(parsed.column, Some(15));
        assert!(
            matches!(parsed.error_type, ErrorType::UndeclaredVariable(ref v) if v == "unknownType")
        );
    }

    #[test]
    fn test_parse_typescript_module_not_found() {
        let error = "index.ts(1,20): error TS2307: Cannot find module 'missing-package'";
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(matches!(parsed.error_type, ErrorType::ModuleNotFound(_)));
    }

    // ==================== Rust Parser Tests ====================

    #[test]
    fn test_parse_rust_undeclared() {
        let error = r#"error[E0425]: cannot find value `undefined_var` in this scope
 --> src/main.rs:10:5
  |
10 |     undefined_var
  |     ^^^^^^^^^^^^^ not found in this scope"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.language, Language::Rust);
        assert_eq!(parsed.file, "src/main.rs");
        assert_eq!(parsed.line, Some(10));
        assert!(
            matches!(parsed.error_type, ErrorType::UndeclaredVariable(ref v) if v == "undefined_var")
        );
    }

    #[test]
    fn test_parse_rust_borrow_error() {
        let error = r#"error[E0502]: cannot borrow `x` as mutable because it is also borrowed as immutable
 --> src/main.rs:5:10
  |
4 |     let r = &x;
  |             -- immutable borrow occurs here"#;
        let result = parse_error(error);

        assert!(result.is_some());
        let parsed = result.unwrap();
        assert!(matches!(parsed.error_type, ErrorType::BorrowError(_)));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_parse_unknown_error() {
        let error = "Some random text that is not an error";
        let result = parse_error(error);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty_input() {
        let result = parse_error("");
        assert!(result.is_none());
    }

    // ==================== Language Display Tests ====================

    #[test]
    fn test_language_display() {
        assert_eq!(format!("{}", Language::Cpp), "C++");
        assert_eq!(format!("{}", Language::Python), "Python");
        assert_eq!(format!("{}", Language::JavaScript), "JavaScript");
        assert_eq!(format!("{}", Language::TypeScript), "TypeScript");
        assert_eq!(format!("{}", Language::Rust), "Rust");
        assert_eq!(format!("{}", Language::Unknown), "Unknown");
    }

    // ==================== ErrorType Equality Tests ====================

    #[test]
    fn test_error_type_equality() {
        assert_eq!(ErrorType::MissingSemicolon, ErrorType::MissingSemicolon);
        assert_eq!(ErrorType::IndentationError, ErrorType::IndentationError);
        assert_ne!(ErrorType::MissingSemicolon, ErrorType::IndentationError);
    }
}
