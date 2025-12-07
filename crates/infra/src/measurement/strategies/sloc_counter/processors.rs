// crates/infra/src/measurement/strategies/sloc_counter/processors.rs
//! 言語別コメント処理モジュール

mod assembly_style;
mod batch_style;
mod c_style;
mod dlang_style;
mod erlang_style;
mod fortran_style;
mod haskell_style;
mod julia_style;
mod lisp_style;
mod lua_style;
mod markup_style;
mod matlab_style;
mod ocaml_style;
mod perl_style;
mod php_style;
mod powershell_style;
mod python_style;
mod ruby_style;
mod simple_hash_style;
mod simple_prefix_style;
mod sql_style;
mod swift_style;
mod vhdl_style;
mod visual_basic_style;

// ============================================================================
// 新しい構造体ベースのプロセッサ (Phase 1+2完了)
// ============================================================================
pub use assembly_style::GasAssemblyProcessor;
pub use c_style::{CStyleProcessor, NestingCStyleProcessor};
pub use dlang_style::DLangProcessor;
pub use haskell_style::HaskellProcessor;
pub use julia_style::JuliaProcessor;
pub use lua_style::LuaProcessor;
pub use markup_style::HtmlProcessor;
pub use matlab_style::MatlabProcessor;
pub use ocaml_style::OCamlProcessor;
pub use perl_style::PerlProcessor;
pub use php_style::PhpProcessor;
pub use powershell_style::PowerShellProcessor;
pub use python_style::PythonProcessor;
pub use ruby_style::RubyProcessor;
pub use simple_hash_style::SimpleHashProcessor;
pub use simple_prefix_style::SimplePrefixProcessor;
pub use sql_style::SqlProcessor;
pub use swift_style::SwiftProcessor;
