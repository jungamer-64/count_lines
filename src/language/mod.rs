pub mod comment_style;
pub mod heredoc_utils;
pub mod processor_trait;
pub mod processors;
pub mod string_utils;

use comment_style::CommentStyle;
pub use processor_trait::LineProcessor;
#[allow(clippy::wildcard_imports)]
use processors::*;
use string_utils::StringSkipOptions;

/// SLOCプロセッサ (Enum Dispatch)
#[derive(Default)]
pub enum SlocProcessor {
    /// C系言語 (//, /* */) - ネストなし
    CStyle(CStyleProcessor),
    /// C系言語 (//, /* */) - ネスト対応 (Rust, Kotlin, Scala)
    NestingCStyle(NestingCStyleProcessor),
    /// JavaScript/TypeScript (Template Literals, Regex)
    JavaScript(JavaScriptProcessor),
    /// Swift (拡張デリミタ文字列対応 + ネストコメント)
    Swift(SwiftProcessor),
    /// Python (Docstring, f-string)
    Python(PythonProcessor),
    /// Ruby (=begin/=end)
    Ruby(RubyProcessor),
    /// Perl (POD)
    Perl(PerlProcessor),
    /// PHP (//, /* */, #)
    Php(PhpProcessor),
    /// `PowerShell` (# と <# #>)
    PowerShell(PowerShellProcessor),
    /// Lua (-- と --[[ ]])
    Lua(LuaProcessor),
    /// HTML/XML (<!-- -->)
    Html(HtmlProcessor),
    /// SQL (-- と /* */)
    Sql(SqlProcessor),
    /// Haskell (-- と {- -})
    Haskell(HaskellProcessor),
    /// Julia (# と #= =#)
    Julia(JuliaProcessor),
    /// OCaml/F#/Pascal ((* *))
    OCaml(OCamlProcessor),
    /// D言語 (//, /* */, /+ +/)
    DLang(DLangProcessor),
    /// MATLAB/Octave (% と %{ %})
    Matlab(MatlabProcessor),
    /// GAS Assembly (# と /* */)
    GasAssembly(GasAssemblyProcessor),
    /// 単純な行コメント (#) (Shell, YAML, etc.)
    SimpleHash(SimpleHashProcessor),
    /// Shell (Here-doc対応)
    Shell(ShellProcessor),
    /// 単純なプレフィックス型コメント (VHDL, Erlang, Lisp, Batch, etc.)
    SimplePrefix(SimplePrefixProcessor),
    /// Fortran (! と C/c/* カラム1)
    Fortran(FortranProcessor),
    /// コメントなし
    #[default]
    NoComment,
}

impl SlocProcessor {
    /// 拡張子からプロセッサを作成
    #[must_use]
    pub fn from_extension(extension: &str) -> Self {
        let style = CommentStyle::from_extension(extension);
        let ext_lower = extension.to_lowercase();
        let string_opts = StringSkipOptions::from_extension(extension);

        match style {
            CommentStyle::CStyle => {
                if ext_lower == "swift" {
                    Self::Swift(SwiftProcessor::new())
                } else if matches!(ext_lower.as_str(), "rs" | "kt" | "kts" | "scala" | "sc") {
                    Self::NestingCStyle(NestingCStyleProcessor::new(string_opts))
                } else if matches!(
                    ext_lower.as_str(),
                    "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts"
                ) {
                    Self::JavaScript(JavaScriptProcessor::new())
                } else {
                    Self::CStyle(CStyleProcessor::new(string_opts))
                }
            }
            CommentStyle::Python => Self::Python(PythonProcessor::default()),
            CommentStyle::Ruby => Self::Ruby(RubyProcessor::default()),
            CommentStyle::Perl => Self::Perl(PerlProcessor::default()),
            CommentStyle::Php => Self::Php(PhpProcessor::new()),
            CommentStyle::PowerShell => Self::PowerShell(PowerShellProcessor::new()),
            CommentStyle::Lua => Self::Lua(LuaProcessor::new()),
            CommentStyle::Html => Self::Html(HtmlProcessor::new()),
            CommentStyle::Sql => Self::Sql(SqlProcessor::new()),
            CommentStyle::Haskell => Self::Haskell(HaskellProcessor::new()),
            CommentStyle::Julia => Self::Julia(JuliaProcessor::new()),
            CommentStyle::OCaml => Self::OCaml(OCamlProcessor::new()),
            CommentStyle::DLang => Self::DLang(DLangProcessor::new()),
            CommentStyle::Matlab => Self::Matlab(MatlabProcessor::new()),
            CommentStyle::GasAssembly => Self::GasAssembly(GasAssemblyProcessor::new()),
            CommentStyle::SimpleHash => {
                if matches!(ext_lower.as_str(), "sh" | "bash" | "zsh") {
                    Self::Shell(ShellProcessor::new())
                } else {
                    Self::SimpleHash(SimpleHashProcessor::default())
                }
            }
            CommentStyle::Vhdl => Self::SimplePrefix(SimplePrefixProcessor::vhdl()),
            CommentStyle::Erlang => Self::SimplePrefix(SimplePrefixProcessor::erlang()),
            CommentStyle::Lisp => Self::SimplePrefix(SimplePrefixProcessor::lisp()),
            CommentStyle::Assembly => Self::SimplePrefix(SimplePrefixProcessor::assembly()),
            CommentStyle::Fortran => Self::Fortran(FortranProcessor::new()),
            CommentStyle::Batch => Self::SimplePrefix(SimplePrefixProcessor::batch()),
            CommentStyle::VisualBasic => Self::SimplePrefix(SimplePrefixProcessor::visual_basic()),
            CommentStyle::None => Self::NoComment,
        }
    }
}

macro_rules! dispatch {
    ($self:expr, $method:ident $(, $args:expr)*) => {
        match $self {
            Self::CStyle(p) => p.$method($($args),*),
            Self::NestingCStyle(p) => p.$method($($args),*),
            Self::JavaScript(p) => p.$method($($args),*),
            Self::Swift(p) => p.$method($($args),*),
            Self::Python(p) => p.$method($($args),*),
            Self::Ruby(p) => p.$method($($args),*),
            Self::Perl(p) => p.$method($($args),*),
            Self::Php(p) => p.$method($($args),*),
            Self::PowerShell(p) => p.$method($($args),*),
            Self::Lua(p) => p.$method($($args),*),
            Self::Html(p) => p.$method($($args),*),
            Self::Sql(p) => p.$method($($args),*),
            Self::Haskell(p) => p.$method($($args),*),
            Self::Julia(p) => p.$method($($args),*),
            Self::OCaml(p) => p.$method($($args),*),
            Self::DLang(p) => p.$method($($args),*),
            Self::Matlab(p) => p.$method($($args),*),
            Self::GasAssembly(p) => p.$method($($args),*),
            Self::SimpleHash(p) => p.$method($($args),*),
            Self::Shell(p) => p.$method($($args),*),
            Self::SimplePrefix(p) => p.$method($($args),*),
            Self::Fortran(p) => p.$method($($args),*),
            Self::NoComment => {
                 // For NoComment, we just count non-empty lines
                 // LineProcessor trait methods might need dummy mapping if mapped directly
                 // But methods are process_line and is_in_block_comment
                 // We handle them specifically in the impl below this macro?
                 // Or we implement methods for NoComment dummy?
                 // Current code handles Self::NoComment specifically in match.
                 panic!("Should not dispatch NoComment via macro if not homogeneous")
            }
        }
    }
}

impl LineProcessor for SlocProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        match self {
            Self::NoComment => usize::from(!line.trim().is_empty()),
            _ => dispatch!(self, process, line),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::NoComment => {}
            _ => dispatch!(self, reset),
        }
    }
}
