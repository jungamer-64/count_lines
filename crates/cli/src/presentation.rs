// crates/cli/src/presentation.rs
use crate::config::Config;
use count_lines_engine::options::{OutputFormat, SortKey, WatchOutput};
use count_lines_engine::stats::FileStats;
use std::cmp::Ordering;
use std::fmt::Write;

pub fn print_clear_screen(output: &WatchOutput) {
    if matches!(output, WatchOutput::Full) {
        print!("\x1B[2J\x1B[1;1H");
    }
}

pub fn print_results(stats: &[FileStats], config: &Config) {
    // Filter out binary files
    let mut stats: Vec<_> = stats.iter().filter(|s| !s.is_binary).cloned().collect();
    if !config.sort.is_empty() {
        stats.sort_by(|a, b| {
            for (key, desc) in &config.sort {
                let order = match key {
                    SortKey::Lines => a.lines.cmp(&b.lines),
                    SortKey::Chars => a.chars.cmp(&b.chars),
                    SortKey::Size => a.size.cmp(&b.size),
                    SortKey::Name => a.name.cmp(&b.name),
                    SortKey::Ext => a.ext.cmp(&b.ext),
                    SortKey::Sloc => a.sloc.unwrap_or(0).cmp(&b.sloc.unwrap_or(0)),
                    SortKey::Words => a.words.unwrap_or(0).cmp(&b.words.unwrap_or(0)),
                };
                if order != Ordering::Equal {
                    return if *desc { order.reverse() } else { order };
                }
            }
            Ordering::Equal
        });
    }

    match config.format {
        OutputFormat::Json => print_json(&stats),
        OutputFormat::Yaml => print_yaml(&stats),
        OutputFormat::Jsonl => print_jsonl(&stats),
        OutputFormat::Md => print_markdown(&stats, config),
        OutputFormat::Csv => print_sv(&stats, config, ","),
        OutputFormat::Tsv => print_sv(&stats, config, "\t"),
        OutputFormat::Table => print_table(&stats, config),
    }
}

fn print_table(stats: &[FileStats], config: &Config) {
    // Get number of threads for parallel info
    let threads = config.walk.threads;

    // Print version header
    println!("count_lines v{} · parallel={threads}", crate::VERSION);
    println!();

    // Print column header
    if config.count_sloc {
        println!("    LINES            SLOC        CHARACTERS     FILE");
    } else {
        println!("    LINES        CHARACTERS     FILE");
    }
    println!("----------------------------------------------");

    // Print each file
    for s in stats {
        if config.count_sloc {
            println!(
                "{:>9}{:>16}{:>16}      {}",
                s.lines,
                s.sloc.map(|v| v.to_string()).unwrap_or_default(),
                s.chars,
                s.path.display()
            );
        } else {
            println!("{:>9}{:>16}      {}", s.lines, s.chars, s.path.display());
        }
    }

    // Print total
    let total_lines: usize = stats.iter().map(|s| s.lines).sum();
    let total_chars: usize = stats.iter().map(|s| s.chars).sum();
    let total_sloc: usize = stats.iter().filter_map(|s| s.sloc).sum();
    let file_count = stats.len();

    println!("---");
    if config.count_sloc {
        println!(
            "{total_lines:>9}{total_sloc:>16}{total_chars:>16}      TOTAL ({file_count} files)"
        );
    } else {
        println!("{total_lines:>9}{total_chars:>16}      TOTAL ({file_count} files)");
    }

    // Print completion message
    println!();
    println!("[count_lines] Completed: {file_count} files processed.");
}

fn print_json(stats: &[FileStats]) {
    if let Ok(json) = serde_json::to_string_pretty(stats) {
        println!("{json}");
    }
}

fn print_yaml(stats: &[FileStats]) {
    if let Ok(yaml) = serde_yaml::to_string(stats) {
        println!("{yaml}");
    }
}

fn print_jsonl(stats: &[FileStats]) {
    let version = crate::VERSION;
    for s in stats {
        if let Ok(mut v) = serde_json::to_value(s) {
            if let Some(obj) = v.as_object_mut() {
                obj.insert("type".to_string(), "file".into());
            }
            println!("{}", serde_json::to_string(&v).unwrap_or_default());
        }
    }

    let total_lines: usize = stats.iter().map(|s| s.lines).sum();
    let total_chars: usize = stats.iter().map(|s| s.chars).sum();
    let total_words: usize = stats.iter().filter_map(|s| s.words).sum();
    let total_sloc: usize = stats.iter().filter_map(|s| s.sloc).sum();
    let file_count = stats.len();

    let total_obj = serde_json::json!({
        "type": "total",
        "version": version,
        "files": file_count,
        "lines": total_lines,
        "chars": total_chars,
        "words": total_words,
        "sloc": total_sloc,
    });
    println!("{total_obj}");
}

fn print_markdown(stats: &[FileStats], config: &Config) {
    println!("### File Statistics");
    println!();
    let mut header = String::from("| Lines |");
    let mut separator = String::from("|:---:|");

    if config.count_sloc {
        header.push_str(" SLOC |");
        separator.push_str(":---:|");
    }

    header.push_str(" Chars |");
    separator.push_str(":---:|");

    if config.count_words {
        header.push_str(" Words |");
        separator.push_str(":---:|");
    }

    header.push_str(" File |");
    separator.push_str(":---|");

    println!("{header}");
    println!("{separator}");

    for s in stats {
        let mut row = format!("| {} |", s.lines);

        if config.count_sloc {
            write!(row, " {} |", s.sloc.unwrap_or(0)).unwrap();
        }

        write!(row, " {} |", s.chars).unwrap();

        if config.count_words {
            write!(row, " {} |", s.words.unwrap_or(0)).unwrap();
        }

        let path_str = s.path.display().to_string().replace('|', "\\|");
        write!(row, " {path_str} |").unwrap();

        println!("{row}");
    }
    println!();
}

fn print_sv(stats: &[FileStats], config: &Config, delimiter: &str) {
    let mut header = String::from("lines");
    if config.count_sloc {
        header.push_str(delimiter);
        header.push_str("sloc");
    }
    header.push_str(delimiter);
    header.push_str("chars");

    if config.count_words {
        header.push_str(delimiter);
        header.push_str("words");
    }

    header.push_str(delimiter);
    header.push_str("path");
    println!("{header}");

    for s in stats {
        let mut row = format!("{}", s.lines);

        if config.count_sloc {
            row.push_str(delimiter);
            row.push_str(&s.sloc.unwrap_or(0).to_string());
        }

        row.push_str(delimiter);
        row.push_str(&s.chars.to_string());

        if config.count_words {
            row.push_str(delimiter);
            row.push_str(&s.words.unwrap_or(0).to_string());
        }

        row.push_str(delimiter);
        let path = s.path.display().to_string();
        if delimiter == "," && (path.contains(',') || path.contains('"') || path.contains('\n')) {
            let escaped = path.replace('"', "\"\"");
            write!(row, "\"{escaped}\"").unwrap();
        } else {
            row.push_str(&path);
        }

        println!("{row}");
    }

    if config.total_row {
        let total_lines: usize = stats.iter().map(|s| s.lines).sum();
        let total_sloc: usize = stats.iter().filter_map(|s| s.sloc).sum();
        let total_chars: usize = stats.iter().map(|s| s.chars).sum();
        let total_words: usize = stats.iter().filter_map(|s| s.words).sum();

        let mut row = format!("{total_lines}");
        if config.count_sloc {
            row.push_str(delimiter);
            row.push_str(&total_sloc.to_string());
        }

        row.push_str(delimiter);
        row.push_str(&total_chars.to_string());

        if config.count_words {
            row.push_str(delimiter);
            row.push_str(&total_words.to_string());
        }

        row.push_str(delimiter);
        row.push_str("TOTAL");
        println!("{row}");
    }
}
