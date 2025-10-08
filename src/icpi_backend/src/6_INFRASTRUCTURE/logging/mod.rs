//! Logging module

/// Log an operation
pub fn log_operation(operation: &str, details: &str) {
    ic_cdk::println!("[{}] {}", operation, details);
}
