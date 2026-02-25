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
    /// `PowerShell`: # と <# #>
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
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // C系言語 (// と /* */)
            "c" | "h" | "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" | "h++" | "cs"
            | "java" | "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" | "mts" | "cts" | "rs"
            | "go" | "swift" | "kt" | "kts" | "scala" | "sc" | "dart" | "v" | "sv" | "svh"
            | "zig" | "m" | "mm" | "groovy" | "gradle" | "css" | "scss" | "sass" | "less"
            | "json" | "jsonc" | "proto" | "thrift" | "sol" | "ld" | "lds" => Self::CStyle,

            // D言語 (//, /* */, /+ +/)
            "d" => Self::DLang,

            // PHP (//, /* */, #)
            "php" => Self::Php,

            // Python: # と Docstring
            "py" | "pyw" | "pyi" => Self::Python,

            // Ruby: # と =begin/=end
            "rb" | "rake" | "gemspec" | "cr" => Self::Ruby,

            // Perl: # と POD
            "pl" | "pm" | "perl" => Self::Perl,

            // 単純な Hash スタイル (#)
            "sh" | "bash" | "zsh" | "fish" | "yml" | "yaml" | "toml" | "dockerfile"
            | "makefile" | "mk" | "cmake" | "nim" | "ex" | "exs" | "coffee" | "tcl" | "awk"
            | "sed" | "tf" | "tfvars" | "r" | "rmd" | "ini" | "conf" | "cfg" | "properties"
            | "graphql" | "gql" | "nix" => Self::SimpleHash,

            // PowerShell (# と <# #>)
            "ps1" | "psm1" | "psd1" => Self::PowerShell,

            // Lua (-- と --[[ ]])
            "lua" => Self::Lua,

            // HTML/XML (<!-- -->)
            "html" | "htm" | "xhtml" | "xml" | "xsl" | "xslt" | "xsd" | "svg" | "vue" => Self::Html,

            // SQL (-- と /* */)
            "sql" => Self::Sql,

            // Haskell (-- と {- -})
            "hs" | "lhs" | "elm" | "purs" => Self::Haskell,

            // Julia (# と #= =#)
            "jl" => Self::Julia,

            // OCaml/F#/Pascal (* *)
            "ml" | "mli" | "fs" | "fsi" | "fsx" | "fsscript" | "pas" | "pp" | "dpr" | "dpk"
            | "sml" | "sig" | "fun" => Self::OCaml,

            // Lisp系 (;)
            "lisp" | "lsp" | "cl" | "el" | "clj" | "cljs" | "cljc" | "edn" | "scm" | "ss"
            | "rkt" => Self::Lisp,

            // Erlang/Elixirのerlang (%) / LaTeX
            "erl" | "hrl" | "tex" | "sty" | "bib" | "ltx" => Self::Erlang,

            // Fortran (!)
            "f" | "f90" | "f95" | "f03" | "f08" | "for" | "ftn" => Self::Fortran,

            // MATLAB (% と %{ %})
            "mat" | "mlx" | "oct" => Self::Matlab,

            // Batch (REM と ::)
            "bat" | "cmd" => Self::Batch,

            // Assembly (NASM/MASM) (; コメント)
            "asm" | "nasm" | "masm" | "inc" => Self::Assembly,

            // GAS/AT&T Assembly (# と /* */)
            "s" => Self::GasAssembly,

            // VHDL (-- コメント)
            "vhd" | "vhdl" => Self::Vhdl,

            // Visual Basic / VBA / VBScript (' と REM)
            "vb" | "vbs" | "bas" | "cls" | "frm" => Self::VisualBasic,

            // その他
            _ => Self::None,
        }
    }
}
