pub mod comment_style;
pub mod heredoc_utils;
pub mod processor_trait;
pub mod processors;
pub mod string_utils;

use comment_style::CommentStyle;
pub use processor_trait::{LineProcessor, LineStats};
#[allow(clippy::wildcard_imports)]
use processors::*;
use string_utils::StringSkipOptions;

use alloc::boxed::Box;
use alloc::string::String;
use hashbrown::HashMap;

fn new_box<T: LineProcessor + 'static>(p: T) -> Box<dyn LineProcessor> {
    Box::new(p)
}

/// 拡張子に応じたプロセッサを生成する
#[must_use]
pub fn get_processor(extension: &str, map: &HashMap<String, String>) -> Box<dyn LineProcessor> {
    // マッピングを確認 (なければそのまま)
    let effective_ext = map.get(extension).map(String::as_str).unwrap_or(extension);

    let style = CommentStyle::from_extension(effective_ext);
    let ext_lower = effective_ext.to_lowercase();
    let string_opts = StringSkipOptions::from_extension(effective_ext);

    match style {
        CommentStyle::CStyle => {
            if ext_lower == "swift" {
                new_box(SwiftProcessor::new())
            } else if matches!(ext_lower.as_str(), "rs" | "kt" | "kts" | "scala" | "sc") {
                new_box(NestingCStyleProcessor::new(string_opts))
            } else if matches!(
                ext_lower.as_str(),
                "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts"
            ) {
                new_box(JavaScriptProcessor::new())
            } else {
                new_box(CStyleProcessor::new(string_opts))
            }
        }
        CommentStyle::Python => new_box(PythonProcessor::default()),
        CommentStyle::Ruby => new_box(RubyProcessor::default()),
        CommentStyle::Perl => new_box(PerlProcessor::default()),
        CommentStyle::Php => new_box(PhpProcessor::new()),
        CommentStyle::PowerShell => new_box(PowerShellProcessor::new()),
        CommentStyle::Lua => new_box(LuaProcessor::new()),
        CommentStyle::Html => new_box(HtmlProcessor::new()),
        CommentStyle::Sql => new_box(SqlProcessor::new()),
        CommentStyle::Haskell => new_box(HaskellProcessor::new()),
        CommentStyle::Julia => new_box(JuliaProcessor::new()),
        CommentStyle::OCaml => new_box(OCamlProcessor::new()),
        CommentStyle::DLang => new_box(DLangProcessor::new()),
        CommentStyle::Matlab => new_box(MatlabProcessor::new()),
        CommentStyle::GasAssembly => new_box(GasAssemblyProcessor::new()),
        CommentStyle::SimpleHash => {
            if matches!(ext_lower.as_str(), "sh" | "bash" | "zsh") {
                new_box(ShellProcessor::new())
            } else {
                new_box(SimpleHashProcessor::default())
            }
        }
        CommentStyle::Vhdl => new_box(SimplePrefixProcessor::vhdl()),
        CommentStyle::Erlang => new_box(SimplePrefixProcessor::erlang()),
        CommentStyle::Lisp => new_box(SimplePrefixProcessor::lisp()),
        CommentStyle::Assembly => new_box(SimplePrefixProcessor::assembly()),
        CommentStyle::Fortran => new_box(FortranProcessor::new()),
        CommentStyle::Batch => new_box(SimplePrefixProcessor::batch()),
        CommentStyle::VisualBasic => new_box(SimplePrefixProcessor::visual_basic()),
        CommentStyle::None => new_box(NoCommentProcessor),
    }
}

/// コメントなしのプロセッサ
struct NoCommentProcessor;

impl LineProcessor for NoCommentProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        usize::from(!line.trim().is_empty())
    }
}
