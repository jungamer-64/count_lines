// crates/infra/src/measurement/strategies/sloc_counter/comment_style.rs
//! コメント構文の種類定義

/// コメント構文の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
    /// C系言語: // と /* */
    CStyle,
    /// PHP: //, /* */, # (全てサポート)
    Php,
    /// Python: # と """...""" / '''...''' Docstring
    Python,
    /// Ruby: # と =begin ～ =end 埋め込みドキュメント
    Ruby,
    /// Perl: # と =pod/=head 等 ～ =cut POD
    Perl,
    /// 単純な Hash スタイル (#) - Shell, YAML, Config系等
    /// 複雑な文字列処理不要、# のみでコメント判定
    SimpleHash,
    /// PowerShell: # と <# #>
    PowerShell,
    /// Lua: -- と --[[ ]]
    Lua,
    /// HTML/XML: <!-- -->
    Html,
    /// SQL: -- と /* */
    Sql,
    /// Haskell: -- と {- -} (ネスト対応)
    Haskell,
    /// Lisp系: ;
    Lisp,
    /// Erlang: %
    Erlang,
    /// Fortran: ! (行頭)
    Fortran,
    /// MATLAB/Octave: % と %{ %}
    Matlab,
    /// Julia: # と #= =# (ネスト対応)
    Julia,
    /// OCaml/F#/Pascal: (* *) (ネスト対応)
    OCaml,
    /// D言語: //, /* */, /+ +/ (ネスト対応)
    DLang,
    /// Batch: REM と ::
    Batch,
    /// Assembly (NASM/MASM): ; のみ
    Assembly,
    /// GAS/AT&T Assembly: # と /* */ (C系に近い)
    GasAssembly,
    /// VHDL: -- のみ (ブロックコメントなし)
    Vhdl,
    /// Visual Basic/VBA/VBS: ' と REM
    VisualBasic,
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
            "v" | "sv" | "svh" => Self::CStyle, // V言語 / Verilog / SystemVerilog
            "zig" => Self::CStyle,  // Zig
            "d" => Self::DLang,     // D言語 (//, /* */, /+ +/)
            "m" | "mm" => Self::CStyle, // Objective-C
            "groovy" | "gradle" => Self::CStyle,
            "css" | "scss" | "sass" | "less" => Self::CStyle,
            "json" | "jsonc" => Self::CStyle, // JSONCはコメント可
            
            // Protocol Buffers, Thrift, Solidity (Cスタイル)
            "proto" => Self::CStyle,
            "thrift" => Self::CStyle,
            "sol" => Self::CStyle, // Solidity
            
            // Linker Script (OS開発で必須)
            "ld" | "lds" => Self::CStyle,
            
            // PHP (//, /* */, #)
            "php" => Self::Php,
            
            // Python: # と Docstring (複雑な文字列処理が必要)
            "py" | "pyw" | "pyi" => Self::Python,
            
            // Ruby: # と =begin/=end
            "rb" | "rake" | "gemspec" => Self::Ruby,
            
            // Perl: # と POD
            "pl" | "pm" | "perl" => Self::Perl,
            
            // 単純な Hash スタイル (#) - 複雑な文字列処理不要
            "sh" | "bash" | "zsh" | "fish" => Self::SimpleHash,
            "yml" | "yaml" => Self::SimpleHash,
            "toml" => Self::SimpleHash,
            "dockerfile" => Self::SimpleHash,
            "makefile" | "mk" => Self::SimpleHash,
            "cmake" => Self::SimpleHash,
            "nim" => Self::SimpleHash, // Nim (# と """ のみ)
            "cr" => Self::Ruby,  // Crystal (Ruby風)
            "ex" | "exs" => Self::SimpleHash, // Elixir (# と @doc/@moduledoc)
            "coffee" => Self::SimpleHash, // CoffeeScript
            "tcl" => Self::SimpleHash,
            "awk" => Self::SimpleHash,
            "sed" => Self::SimpleHash,
            "tf" | "tfvars" => Self::SimpleHash, // Terraform
            "r" | "rmd" => Self::SimpleHash, // R
            
            // 設定ファイル (#) - 単純処理
            "ini" | "conf" | "cfg" | "properties" => Self::SimpleHash,
            
            // GraphQL (#) - 単純処理
            "graphql" | "gql" => Self::SimpleHash,
            
            // PowerShell (# と <# #>)
            "ps1" | "psm1" | "psd1" => Self::PowerShell,
            "nix" => Self::SimpleHash, // Nix
            
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
            
            // Julia (# と #= =#)
            "jl" => Self::Julia,
            
            // OCaml/F#/Pascal (* *)
            "ml" | "mli" => Self::OCaml, // OCaml
            "fs" | "fsi" | "fsx" | "fsscript" => Self::OCaml, // F#
            "pas" | "pp" | "dpr" | "dpk" => Self::OCaml, // Pascal/Delphi (inc は曖昧)
            "sml" | "sig" | "fun" => Self::OCaml, // Standard ML
            
            // Lisp系 (;)
            "lisp" | "lsp" | "cl" => Self::Lisp,
            "el" => Self::Lisp,  // Emacs Lisp
            "clj" | "cljs" | "cljc" | "edn" => Self::Lisp, // Clojure
            "scm" | "ss" | "rkt" => Self::Lisp, // Scheme, Racket
            
            // Erlang/Elixirのerlang (%)
            "erl" | "hrl" => Self::Erlang,
            
            // LaTeX (論文・ドキュメント)
            "tex" | "sty" | "bib" | "ltx" => Self::Erlang,
            
            // Fortran (!)
            "f" | "f90" | "f95" | "f03" | "f08" | "for" | "ftn" => Self::Fortran,
            
            // MATLAB (% と %{ %})
            // 注: ".m" はObjective-Cとして扱う（より一般的）
            "mat" | "mlx" => Self::Matlab,
            "oct" => Self::Matlab, // Octave
            
            // Batch (REM と ::)
            "bat" | "cmd" => Self::Batch,
            
            // Assembly - NASM/MASM (; コメント)
            "asm" | "nasm" | "masm" | "inc" => Self::Assembly,
            
            // Assembly - GAS/AT&T (# と /* */ コメント)
            "s" => Self::GasAssembly,
            
            // VHDL (-- コメント)
            "vhd" | "vhdl" => Self::Vhdl,
            
            // Visual Basic / VBA / VBScript (' と REM)
            // cls は VBA クラスモジュール、frm はフォームモジュール、bas は標準モジュール
            "vb" | "vbs" | "bas" | "cls" | "frm" => Self::VisualBasic,
            
            // その他（コメント構文なし）
            _ => Self::None,
        }
    }
}
