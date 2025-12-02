use count_lines_infra::measurement::strategies::SlocCounter;

fn main() {
    let mut counter = SlocCounter::new("rs");
    
    let lines = vec![
        "//! Security Policy Engine for ExoRust",
        "//!",
        "//! This module implements a flexible rule-based security policy",
        "//! system for controlling access and operations.",
        "",
        "use core::fmt;",
        "use alloc::vec::Vec;",
        "",
        "/// Policy action to take",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
        "pub enum PolicyAction {",
        "    /// Allow the operation",
        "    Allow,",
        "    /// Deny the operation", 
        "    Deny,",
        "}",
    ];
    
    for line in &lines {
        counter.process_line(line);
        println!("After '{}': count = {}", line, counter.count());
    }
    
    println!("Final count: {}", counter.count());
}
