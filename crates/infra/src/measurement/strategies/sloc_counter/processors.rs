// crates/infra/src/measurement/strategies/sloc_counter/processors.rs
//! 言語別コメント処理モジュール

mod assembly_style;
mod batch_style;
mod c_style;
mod erlang_style;
mod fortran_style;
mod hash_style;
mod haskell_style;
mod julia_style;
mod lisp_style;
mod lua_style;
mod markup_style;
mod matlab_style;
mod ocaml_style;
mod php_style;
mod powershell_style;
mod python_style;
mod sql_style;
mod visual_basic_style;

// フラットにエクスポート
pub use assembly_style::{process_assembly_style, process_gas_assembly_style, process_vhdl_style};
pub use batch_style::process_batch_style;
pub use c_style::{
    process_c_style_with_options, process_dlang_style,
    process_nesting_c_style_with_options, process_swift_style,
};
pub use erlang_style::process_erlang_style;
pub use fortran_style::process_fortran_style;
pub use hash_style::{
    process_perl_style, process_ruby_style, process_simple_hash_style,
};
pub use haskell_style::process_haskell_style;
pub use julia_style::process_julia_style;
pub use lisp_style::process_lisp_style;
pub use lua_style::process_lua_style;
pub use markup_style::process_html_style;
pub use matlab_style::process_matlab_style;
pub use ocaml_style::process_ocaml_style;
pub use php_style::process_php_style;
pub use powershell_style::process_powershell_style;
pub use python_style::process_python_style;
pub use sql_style::process_sql_style;
pub use visual_basic_style::process_visual_basic_style;
