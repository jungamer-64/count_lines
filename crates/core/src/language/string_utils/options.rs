// crates/core/src/language/string_utils/options.rs
/// Options controlling which string literal syntaxes to recognize.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StringSkipOptions {
    flags: u16,
}

impl StringSkipOptions {
    const RUST_RAW_STRING: u16 = 1 << 0;
    const RUST_BYTE_STRING: u16 = 1 << 1;
    const RUST_LIFETIME: u16 = 1 << 2;
    const CPP_RAW_STRING: u16 = 1 << 3;
    const CSHARP_VERBATIM: u16 = 1 << 4;
    const TEXT_BLOCK: u16 = 1 << 5;
    const BACKTICK_STRING: u16 = 1 << 6;
    const DOUBLE_QUOTE: u16 = 1 << 7;
    const SINGLE_QUOTE: u16 = 1 << 8;
    const REGEX_LITERAL: u16 = 1 << 9;

    /// Returns `true` if Rust raw string literals (`r"..."`) are enabled.
    #[must_use]
    pub const fn rust_raw_string(self) -> bool {
        self.flags & Self::RUST_RAW_STRING != 0
    }
    /// Returns `true` if Rust byte string literals (`b"..."`) are enabled.
    #[must_use]
    pub const fn rust_byte_string(self) -> bool {
        self.flags & Self::RUST_BYTE_STRING != 0
    }
    /// Returns `true` if Rust lifetime annotation handling (`'a`) is enabled.
    #[must_use]
    pub const fn rust_lifetime(self) -> bool {
        self.flags & Self::RUST_LIFETIME != 0
    }
    /// Returns `true` if C++ raw string literals (`R"(...)"`) are enabled.
    #[must_use]
    pub const fn cpp_raw_string(self) -> bool {
        self.flags & Self::CPP_RAW_STRING != 0
    }
    /// Returns `true` if C# verbatim strings (`@"..."`) are enabled.
    #[must_use]
    pub const fn csharp_verbatim(self) -> bool {
        self.flags & Self::CSHARP_VERBATIM != 0
    }
    /// Returns `true` if text blocks (`"""..."""`) are enabled.
    #[must_use]
    pub const fn text_block(self) -> bool {
        self.flags & Self::TEXT_BLOCK != 0
    }
    /// Returns `true` if backtick strings are enabled.
    #[must_use]
    pub const fn backtick_string(self) -> bool {
        self.flags & Self::BACKTICK_STRING != 0
    }
    /// Returns `true` if double-quote strings are enabled.
    #[must_use]
    pub const fn double_quote(self) -> bool {
        self.flags & Self::DOUBLE_QUOTE != 0
    }
    /// Returns `true` if single-quote strings are enabled.
    #[must_use]
    pub const fn single_quote(self) -> bool {
        self.flags & Self::SINGLE_QUOTE != 0
    }
    /// Returns `true` if regex literals (`/.../`) are enabled.
    #[must_use]
    pub const fn regex_literal(self) -> bool {
        self.flags & Self::REGEX_LITERAL != 0
    }

    /// Sets the given flag bit and returns the modified options.
    #[must_use]
    pub const fn with_flag(mut self, flag: u16) -> Self {
        self.flags |= flag;
        self
    }

    /// Rust 用オプション
    #[must_use]
    pub fn rust() -> Self {
        Self::default()
            .with_flag(Self::RUST_RAW_STRING)
            .with_flag(Self::RUST_BYTE_STRING)
            .with_flag(Self::RUST_LIFETIME)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// C/C++ 用オプション (Raw String対応)
    #[must_use]
    pub fn cpp() -> Self {
        Self::default()
            .with_flag(Self::CPP_RAW_STRING)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// C 用オプション (Raw String なし)
    #[must_use]
    pub fn c() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// C# 用オプション (Verbatim String @"..." 対応)
    #[must_use]
    pub fn csharp() -> Self {
        Self::default()
            .with_flag(Self::CSHARP_VERBATIM)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// Java/Kotlin/Scala 用オプション (Text Block """...""" 対応)
    #[must_use]
    pub fn java_kotlin() -> Self {
        Self::default()
            .with_flag(Self::TEXT_BLOCK)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// Go 用オプション (バッククォート `...` 対応、正規表現リテラルなし)
    #[must_use]
    pub fn go() -> Self {
        Self::default()
            .with_flag(Self::BACKTICK_STRING)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// JavaScript/TypeScript 用オプション (バッククォート `...` と正規表現 /.../ 対応)
    #[must_use]
    pub fn javascript() -> Self {
        Self::default()
            .with_flag(Self::BACKTICK_STRING)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
            .with_flag(Self::REGEX_LITERAL)
    }

    /// Ruby 用オプション (正規表現 /.../ 対応)
    #[must_use]
    pub fn ruby() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
            .with_flag(Self::REGEX_LITERAL)
    }

    /// Perl 用オプション (正規表現 /.../ 対応)
    #[must_use]
    pub fn perl() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
            .with_flag(Self::REGEX_LITERAL)
    }

    /// Swift 用オプション (拡張デリミタ #"..."# 対応)
    ///
    /// Swift固有の文字列:
    /// - 通常: `"..."`
    /// - 多重引用符: `"""..."""`
    /// - 拡張デリミタ: `#"..."#`, `##"..."##`
    #[must_use]
    pub fn swift() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// Verilog/SystemVerilog 用オプション
    ///
    /// Verilog は C風の文字列のみ (Raw String なし)
    #[must_use]
    pub fn verilog() -> Self {
        Self::default().with_flag(Self::DOUBLE_QUOTE)
    }

    /// Dart 用オプション
    ///
    /// Dart はバッククォートなし、三重クォートあり
    #[must_use]
    pub fn dart() -> Self {
        Self::default()
            .with_flag(Self::TEXT_BLOCK)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// Objective-C 用オプション
    ///
    /// @"..." 形式の `NSString` リテラルがあるが、
    /// C# Verbatim String とは異なりエスケープ可能
    #[must_use]
    pub fn objc() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// 基本的な C スタイル (多くの言語で共通)
    #[must_use]
    pub fn basic() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// 拡張子から適切なオプションを取得
    #[allow(clippy::wildcard_in_or_patterns)]
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Rust
            "rs" => Self::rust(),

            // C/C++
            "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" | "h++" => Self::cpp(),
            "c" | "h" => Self::c(),

            // C#
            "cs" => Self::csharp(),

            // Java/Kotlin/Scala/Groovy
            "java" | "kt" | "kts" | "scala" | "sc" | "groovy" | "gradle" => Self::java_kotlin(),

            // Go (バッククォート対応、正規表現リテラルなし)
            "go" => Self::go(),

            // JavaScript/TypeScript (バッククォート + 正規表現リテラル対応)
            "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" | "mts" | "cts" => Self::javascript(),

            // Swift
            "swift" => Self::swift(),

            // Dart
            "dart" => Self::dart(),

            // Objective-C
            "m" | "mm" => Self::objc(),

            // Verilog/SystemVerilog
            "v" | "sv" | "svh" => Self::verilog(),

            // Ruby (正規表現リテラル対応)
            "rb" | "rake" | "gemspec" | "podspec" | "jbuilder" | "erb" => Self::ruby(),

            // Perl (正規表現リテラル対応)
            "pl" | "pm" | "t" | "psgi" => Self::perl(),

            // C-Style Basic (D, Zig, Proto, CSS, etc.)
            "d" | "proto" | "thrift" | "sol" | "ld" | "lds" | "zig" | "css" | "scss" | "sass"
            | "less" | _ => Self::basic(),
        }
    }
}
