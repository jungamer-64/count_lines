pub mod byte_based;
pub mod line_based;

pub use byte_based::measure_entire_file;
pub use line_based::measure_by_lines;
