// crates/infra/src/measurement/strategies/sloc_counter/comment_style.rs
//! コメント構文の種類定義

/// コメント構文の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
    /// C系言語: // と /* */
    CStyle,
    /// Python/Ruby/Shell: #
    Hash,
    /// PowerShell: # と <# #>
    PowerShell,
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
            "tf" | "tfvars" => Self::Hash, // Terraform
            
            // PowerShell (# と <# #>)
            "ps1" | "psm1" | "psd1" => Self::PowerShell,
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
