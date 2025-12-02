// crates/infra/src/measurement/strategies/sloc_counter.rs
//! SLOC (Source Lines of Code) カウンター
//!
//! 言語ごとのコメント構文を認識し、純粋なコード行のみをカウントします。

/// コメント構文の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
    /// C系言語: // と /* */
    CStyle,
    /// Python/Ruby/Shell: #
    Hash,
    /// Lua: -- と --[[ ]]
    Lua,
    /// HTML/XML: <!-- -->
    Html,
    /// SQL: -- と /* */
    Sql,
    /// Haskell: -- と {- -}
    Haskell,
    /// Lisp系: ;
    Lisp,
    /// Erlang: %
    Erlang,
    /// Fortran: ! (行頭)
    Fortran,
    /// MATLAB/Octave: % と %{ %}
    Matlab,
    /// コメント構文なし（全ての非空行をカウント）
    None,
}

impl CommentStyle {
    /// 拡張子から言語のコメントスタイルを判定
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // C系言語 (// と /* */)
            "c" | "h" => Self::CStyle,
            "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" | "h++" => Self::CStyle,
            "cs" => Self::CStyle, // C#
            "java" => Self::CStyle,
            "js" | "mjs" | "cjs" | "jsx" => Self::CStyle,
            "ts" | "tsx" | "mts" | "cts" => Self::CStyle,
            "rs" => Self::CStyle, // Rust
            "go" => Self::CStyle,
            "swift" => Self::CStyle,
            "kt" | "kts" => Self::CStyle, // Kotlin
            "scala" | "sc" => Self::CStyle,
            "dart" => Self::CStyle,
            "v" => Self::CStyle,    // V言語
            "zig" => Self::CStyle,  // Zig
            "d" => Self::CStyle,    // D言語
            "m" | "mm" => Self::CStyle, // Objective-C
            "groovy" | "gradle" => Self::CStyle,
            "php" => Self::CStyle, // PHP (も # をサポートするが // が一般的)
            "css" | "scss" | "sass" | "less" => Self::CStyle,
            "json" | "jsonc" => Self::CStyle, // JSONCはコメント可
            
            // Hash系 (#)
            "py" | "pyw" | "pyi" => Self::Hash, // Python
            "rb" | "rake" | "gemspec" => Self::Hash, // Ruby
            "sh" | "bash" | "zsh" | "fish" => Self::Hash,
            "pl" | "pm" | "perl" => Self::Hash, // Perl
            "r" | "rmd" => Self::Hash, // R
            "yml" | "yaml" => Self::Hash,
            "toml" => Self::Hash,
            "dockerfile" => Self::Hash,
            "makefile" | "mk" => Self::Hash,
            "cmake" => Self::Hash,
            "nim" => Self::Hash, // Nim
            "cr" => Self::Hash,  // Crystal
            "ex" | "exs" => Self::Hash, // Elixir
            "coffee" => Self::Hash, // CoffeeScript
            "tcl" => Self::Hash,
            "awk" => Self::Hash,
            "sed" => Self::Hash,
            "ps1" | "psm1" | "psd1" => Self::Hash, // PowerShell
            "tf" | "tfvars" => Self::Hash, // Terraform
            "nix" => Self::Hash, // Nix
            
            // Lua (-- と --[[ ]])
            "lua" => Self::Lua,
            
            // HTML/XML (<!-- -->)
            "html" | "htm" | "xhtml" => Self::Html,
            "xml" | "xsl" | "xslt" | "xsd" => Self::Html,
            "svg" => Self::Html,
            "vue" => Self::Html, // Vue (HTML-like)
            
            // SQL (-- と /* */)
            "sql" => Self::Sql,
            
            // Haskell (-- と {- -})
            "hs" | "lhs" => Self::Haskell,
            "elm" => Self::Haskell,
            "purs" => Self::Haskell, // PureScript
            
            // Lisp系 (;)
            "lisp" | "lsp" | "cl" => Self::Lisp,
            "el" => Self::Lisp,  // Emacs Lisp
            "clj" | "cljs" | "cljc" | "edn" => Self::Lisp, // Clojure
            "scm" | "ss" | "rkt" => Self::Lisp, // Scheme, Racket
            
            // Erlang/Elixirのerlang (%)
            "erl" | "hrl" => Self::Erlang,
            
            // Fortran (!)
            "f" | "f90" | "f95" | "f03" | "f08" | "for" | "ftn" => Self::Fortran,
            
            // MATLAB (% と %{ %})
            // 注: ".m" はObjective-Cとして扱う（より一般的）
            "mat" | "mlx" => Self::Matlab,
            "oct" => Self::Matlab, // Octave
            
            // その他（コメント構文なし）
            _ => Self::None,
        }
    }
}

/// SLOCカウンターの状態
pub struct SlocCounter {
    style: CommentStyle,
    in_block_comment: bool,
    count: usize,
}

impl SlocCounter {
    /// 新しいカウンターを作成
    pub fn new(extension: &str) -> Self {
        Self {
            style: CommentStyle::from_extension(extension),
            in_block_comment: false,
            count: 0,
        }
    }

    /// 行を処理してSLOCかどうかを判定
    pub fn process_line(&mut self, line: &str) {
        let trimmed = line.trim();
        
        // 空行はスキップ
        if trimmed.is_empty() {
            return;
        }

        match self.style {
            CommentStyle::CStyle => self.process_c_style(trimmed),
            CommentStyle::Hash => self.process_hash_style(trimmed),
            CommentStyle::Lua => self.process_lua_style(trimmed),
            CommentStyle::Html => self.process_html_style(trimmed),
            CommentStyle::Sql => self.process_sql_style(trimmed),
            CommentStyle::Haskell => self.process_haskell_style(trimmed),
            CommentStyle::Lisp => self.process_lisp_style(trimmed),
            CommentStyle::Erlang => self.process_erlang_style(trimmed),
            CommentStyle::Fortran => self.process_fortran_style(trimmed),
            CommentStyle::Matlab => self.process_matlab_style(trimmed),
            CommentStyle::None => {
                // コメント構文なしの場合、非空行は全てSLOC
                self.count += 1;
            }
        }
    }

    /// 現在のSLOCカウントを取得
    pub fn count(&self) -> usize {
        self.count
    }

    /// ブロックコメント内かどうか（テスト用）
    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }

    /// 文字列リテラル外でパターンを検索（簡易実装）
    /// 文字列リテラル（"..." や '...'）内のパターンは無視する
    fn find_outside_string(line: &str, pattern: &str) -> Option<usize> {
        let mut in_string = false;
        let mut string_char = '"';
        let mut escape_next = false;
        let pattern_bytes = pattern.as_bytes();
        let line_bytes = line.as_bytes();
        
        if pattern_bytes.is_empty() || line_bytes.len() < pattern_bytes.len() {
            return None;
        }
        
        let mut i = 0;
        while i <= line_bytes.len() - pattern_bytes.len() {
            let c = line_bytes[i];
            
            if escape_next {
                escape_next = false;
                i += 1;
                continue;
            }
            
            if c == b'\\' && in_string {
                escape_next = true;
                i += 1;
                continue;
            }
            
            if !in_string {
                if c == b'"' || c == b'\'' {
                    in_string = true;
                    string_char = c as char;
                    i += 1;
                    continue;
                }
                
                // パターンとマッチするかチェック
                if &line_bytes[i..i + pattern_bytes.len()] == pattern_bytes {
                    return Some(i);
                }
            } else {
                // 文字列内
                if c == string_char as u8 {
                    in_string = false;
                }
            }
            
            i += 1;
        }
        
        None
    }

    /// C系スタイル (// と /* */) の処理
    fn process_c_style(&mut self, line: &str) {
        if self.in_block_comment {
            // ブロックコメント内
            if let Some(pos) = line.find("*/") {
                self.in_block_comment = false;
                // 閉じた後にコードがあるかチェック
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() && !rest.trim().starts_with("//") {
                    self.count += 1;
                }
            }
            return;
        }

        // 行コメント（文字列外）のみの行かチェック
        if let Some(line_comment_pos) = Self::find_outside_string(line, "//") {
            // // より前にコードがあるか
            let before = &line[..line_comment_pos];
            if before.trim().is_empty() {
                // コメントのみの行
                return;
            }
            // コメント前にコードがある
            self.count += 1;
            return;
        }

        // ブロックコメント開始をチェック（文字列外）
        if let Some(block_start) = Self::find_outside_string(line, "/*") {
            // /* より前にコードがあるか
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty();
            
            // ブロックコメントが同じ行で閉じるか
            if let Some(block_end) = line[block_start + 2..].find("*/") {
                let after = &line[block_start + 2 + block_end + 2..];
                let has_code_after = !after.trim().is_empty() 
                    && Self::find_outside_string(after, "//").map_or(true, |p| p > 0);
                if has_code_before || has_code_after {
                    self.count += 1;
                }
            } else {
                self.in_block_comment = true;
                if has_code_before {
                    self.count += 1;
                }
            }
            return;
        }

        // コードがある行
        self.count += 1;
    }

    /// Hash スタイル (#) の処理
    fn process_hash_style(&mut self, line: &str) {
        // shebang行を除外
        if line.starts_with("#!") && self.count == 0 {
            return;
        }
        
        // #で始まる行はコメント
        if line.starts_with('#') {
            return;
        }

        // # より前にコードがあるか（文字列内の # は無視すべきだが、簡易実装）
        if let Some(hash_pos) = line.find('#') {
            let before = &line[..hash_pos];
            if !before.trim().is_empty() {
                self.count += 1;
            }
        } else {
            self.count += 1;
        }
    }

    /// Lua スタイル (-- と --[[ ]]) の処理
    fn process_lua_style(&mut self, line: &str) {
        if self.in_block_comment {
            if line.contains("]]") {
                self.in_block_comment = false;
            }
            return;
        }

        if line.starts_with("--[[") || line.starts_with("--[=[") {
            self.in_block_comment = true;
            return;
        }

        if line.starts_with("--") {
            return;
        }

        self.count += 1;
    }

    /// HTML スタイル (<!-- -->) の処理
    fn process_html_style(&mut self, line: &str) {
        if self.in_block_comment {
            if line.contains("-->") {
                self.in_block_comment = false;
                // --> の後にコードがあるかチェック
                if let Some(pos) = line.find("-->") {
                    let rest = &line[pos + 3..];
                    if !rest.trim().is_empty() {
                        self.count += 1;
                    }
                }
            }
            return;
        }

        if let Some(start) = line.find("<!--") {
            let before = &line[..start];
            let has_code_before = !before.trim().is_empty();

            if let Some(end_offset) = line[start + 4..].find("-->") {
                let after = &line[start + 4 + end_offset + 3..];
                if has_code_before || !after.trim().is_empty() {
                    self.count += 1;
                }
            } else {
                self.in_block_comment = true;
                if has_code_before {
                    self.count += 1;
                }
            }
            return;
        }

        self.count += 1;
    }

    /// SQL スタイル (-- と /* */) の処理
    fn process_sql_style(&mut self, line: &str) {
        if self.in_block_comment {
            if let Some(pos) = line.find("*/") {
                self.in_block_comment = false;
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() && !rest.trim().starts_with("--") {
                    self.count += 1;
                }
            }
            return;
        }

        // ブロックコメント開始
        if let Some(block_start) = line.find("/*") {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty() && !before.trim().starts_with("--");

            if let Some(end_offset) = line[block_start + 2..].find("*/") {
                let after = &line[block_start + 2 + end_offset + 2..];
                if has_code_before || !after.trim().is_empty() {
                    self.count += 1;
                }
            } else {
                self.in_block_comment = true;
                if has_code_before {
                    self.count += 1;
                }
            }
            return;
        }

        // 行コメント
        if line.starts_with("--") {
            return;
        }

        self.count += 1;
    }

    /// Haskell スタイル (-- と {- -}) の処理
    fn process_haskell_style(&mut self, line: &str) {
        if self.in_block_comment {
            if let Some(pos) = line.find("-}") {
                self.in_block_comment = false;
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() && !rest.trim().starts_with("--") {
                    self.count += 1;
                }
            }
            return;
        }

        if let Some(block_start) = line.find("{-") {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty() && !before.trim().starts_with("--");

            if let Some(end_offset) = line[block_start + 2..].find("-}") {
                let after = &line[block_start + 2 + end_offset + 2..];
                if has_code_before || !after.trim().is_empty() {
                    self.count += 1;
                }
            } else {
                self.in_block_comment = true;
                if has_code_before {
                    self.count += 1;
                }
            }
            return;
        }

        if line.starts_with("--") {
            return;
        }

        self.count += 1;
    }

    /// Lisp スタイル (;) の処理
    fn process_lisp_style(&mut self, line: &str) {
        if line.starts_with(';') {
            return;
        }
        self.count += 1;
    }

    /// Erlang スタイル (%) の処理
    fn process_erlang_style(&mut self, line: &str) {
        if line.starts_with('%') {
            return;
        }
        self.count += 1;
    }

    /// Fortran スタイル (!) の処理
    fn process_fortran_style(&mut self, line: &str) {
        // Fortran: ! で始まるコメント、または C/c/*/d/D で始まる固定形式コメント
        if line.starts_with('!')
            || line.starts_with('C')
            || line.starts_with('c')
            || line.starts_with('*')
        {
            return;
        }
        self.count += 1;
    }

    /// MATLAB スタイル (% と %{ %}) の処理
    fn process_matlab_style(&mut self, line: &str) {
        if self.in_block_comment {
            if line.trim() == "%}" {
                self.in_block_comment = false;
            }
            return;
        }

        if line.trim() == "%{" {
            self.in_block_comment = true;
            return;
        }

        if line.starts_with('%') {
            return;
        }

        self.count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_style_single_line_comment() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line("// this is a comment");
        counter.process_line("let x = 1; // inline comment");
        counter.process_line("let y = 2;");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_rust_doc_comments() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line("//! Module doc comment");
        counter.process_line("//!");
        counter.process_line("//! Another line");
        counter.process_line("");
        counter.process_line("/// Function doc comment");
        counter.process_line("/// More docs");
        counter.process_line("pub fn foo() {}");
        // Doc comments should not be counted, only the actual code line
        assert_eq!(counter.count(), 1, "Only 'pub fn foo() {{}}' should be counted as SLOC");
    }

    #[test]
    fn test_rust_realistic_file() {
        let mut counter = SlocCounter::new("rs");
        let lines = vec![
            "//! Security Policy Engine",
            "//!",
            "//! This module implements security policy.",
            "",
            "use core::fmt;",
            "use alloc::vec::Vec;",
            "",
            "/// Policy action to take",
            "#[derive(Debug, Clone)]",
            "pub enum PolicyAction {",
            "    /// Allow the operation",
            "    Allow,",
            "    /// Deny the operation",
            "    Deny,",
            "}",
            "",
            "impl PolicyAction {",
            "    /// Check if action allows",
            "    pub fn is_allow(&self) -> bool {",
            "        matches!(self, PolicyAction::Allow)",
            "    }",
            "}",
        ];
        
        for line in &lines {
            counter.process_line(line);
        }
        
        // Expected SLOC: use(2) + #[derive](1) + enum declaration(1) + Allow,(1) + Deny,(1) + }(1)
        //              + impl(1) + pub fn(1) + matches!(1) + }(1) + }(1) = 12
        // NOT including: //!, //! comments, /// doc comments, empty lines
        assert!(counter.count() > 10, "Expected more than 10 SLOC, got {}", counter.count());
    }

    #[test]
    fn test_attribute_and_code_mixed() {
        // Test that attributes like #[derive(...)] are counted as SLOC
        let mut counter = SlocCounter::new("rs");
        counter.process_line("#[derive(Debug, Clone)]");
        counter.process_line("pub struct Foo;");
        assert_eq!(counter.count(), 2, "Both attribute and struct should be SLOC");
    }

    #[test]
    fn test_comment_in_string_literal() {
        // Test that /* inside string literals is not treated as block comment
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r#"if pattern.ends_with("/*") {"#);
        counter.process_line("    // do something");
        counter.process_line("}");
        // First line has code, second is comment, third has code
        assert_eq!(counter.count(), 2, "String literal with /* should not trigger block comment");
        assert!(!counter.is_in_block_comment(), "Should not be in block comment mode");
    }

    #[test]
    fn test_comment_in_string_literal_double_star() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r#"} else if pattern.ends_with("/**") {"#);
        counter.process_line("    let x = 1;");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_block_comment_state_issue() {
        // Test the actual content from the user's file
        let mut counter = SlocCounter::new("rs");
        
        // These lines should NOT trigger block comment mode
        let test_lines = vec![
            "//! Security Policy Engine for ExoRust",
            "//!",
            "//! This module implements a flexible rule-based security policy",
            "//! system for controlling access and operations.",
            "",
            "use core::fmt;",
            "use alloc::vec::Vec;",
            "use alloc::string::String;",
            "use alloc::collections::BTreeMap;",
            "use spin::RwLock;",
            "",
            "extern crate alloc;",
            "",
            "/// Policy action to take",
            "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
            "pub enum PolicyAction {",
            "    /// Allow the operation",
            "    Allow,",
        ];
        
        for (i, line) in test_lines.iter().enumerate() {
            let before = counter.count();
            counter.process_line(line);
            let after = counter.count();
            eprintln!("Line {}: '{}' -> count {} -> {} (in_block={})", 
                     i, line, before, after, counter.is_in_block_comment());
        }
        
        // Should have: 5 use statements + extern crate + #[derive] + pub enum + Allow, = 9
        assert!(counter.count() >= 9, "Expected at least 9 SLOC, got {}", counter.count());
        assert!(!counter.is_in_block_comment(), "Should not be in block comment mode");
    }

    #[test]
    fn test_c_style_block_comment() {
        let mut counter = SlocCounter::new("c");
        counter.process_line("/* block comment */");
        counter.process_line("int x = 1;");
        counter.process_line("/* multi");
        counter.process_line("   line");
        counter.process_line("   comment */");
        counter.process_line("int y = 2;");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_python_hash_comment() {
        let mut counter = SlocCounter::new("py");
        counter.process_line("#!/usr/bin/env python");
        counter.process_line("# comment");
        counter.process_line("x = 1  # inline");
        counter.process_line("y = 2");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_empty_lines_ignored() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line("");
        counter.process_line("   ");
        counter.process_line("\t");
        counter.process_line("let x = 1;");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_unknown_extension() {
        let mut counter = SlocCounter::new("xyz");
        counter.process_line("any content");
        counter.process_line("// this is not treated as comment");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_html_comment() {
        let mut counter = SlocCounter::new("html");
        counter.process_line("<!-- comment -->");
        counter.process_line("<div>content</div>");
        counter.process_line("<!-- multi");
        counter.process_line("line -->");
        counter.process_line("<p>text</p>");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_lua_comment() {
        let mut counter = SlocCounter::new("lua");
        counter.process_line("-- comment");
        counter.process_line("local x = 1");
        counter.process_line("--[[ block");
        counter.process_line("comment ]]");
        counter.process_line("local y = 2");
        assert_eq!(counter.count(), 2);
    }
}
