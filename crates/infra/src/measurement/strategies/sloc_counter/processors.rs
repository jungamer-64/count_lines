// crates/infra/src/measurement/strategies/sloc_counter/processors.rs
//! 言語別コメント処理モジュール

mod c_style;
mod hash_style;
mod julia_style;
mod ocaml_style;
mod other_styles;
mod php_style;
mod powershell_style;

pub use c_style::{
    process_c_style, process_cpp_style, process_dlang_style, process_nesting_c_style,
    process_swift_style,
};
pub use hash_style::process_hash_style;
pub use julia_style::process_julia_style;
pub use ocaml_style::process_ocaml_style;
pub use other_styles::{
    process_assembly_style, process_batch_style, process_erlang_style, process_fortran_style,
    process_gas_assembly_style, process_haskell_style, process_html_style, process_lisp_style,
    process_lua_style, process_matlab_style, process_sql_style, process_vhdl_style,
    process_visual_basic_style,
};
pub use php_style::process_php_style;
pub use powershell_style::process_powershell_style;
