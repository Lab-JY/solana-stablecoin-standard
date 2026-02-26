use colored::Colorize;

/// Print a success message with a green checkmark.
pub fn success(msg: &str) {
    println!("{} {}", "✔".green().bold(), msg);
}

/// Print an error message with a red cross.
pub fn error(msg: &str) {
    eprintln!("{} {}", "✖".red().bold(), msg);
}

/// Print an info message with a blue arrow.
pub fn info(msg: &str) {
    println!("{} {}", "→".blue().bold(), msg);
}

/// Print a warning message with a yellow exclamation.
pub fn warn(msg: &str) {
    println!("{} {}", "!".yellow().bold(), msg);
}

/// Print a labeled key-value pair.
pub fn field(label: &str, value: &str) {
    println!("  {}: {}", label.dimmed(), value.white().bold());
}

/// Print a section header.
pub fn header(title: &str) {
    println!("\n{}", title.cyan().bold().underline());
}

/// Print a transaction signature.
pub fn tx_signature(sig: &str) {
    println!("\n{} {}", "Transaction:".dimmed(), sig.yellow());
}

/// Print a divider line.
pub fn divider() {
    println!("{}", "─".repeat(60).dimmed());
}

/// Print a table row with fixed-width columns.
pub fn table_row(cols: &[(&str, usize)]) {
    let formatted: Vec<String> = cols
        .iter()
        .map(|(val, width)| format!("{:<width$}", val, width = width))
        .collect();
    println!("  {}", formatted.join("  "));
}

/// Print a table header row.
pub fn table_header(cols: &[(&str, usize)]) {
    let formatted: Vec<String> = cols
        .iter()
        .map(|(val, width)| format!("{:<width$}", val.to_uppercase(), width = width))
        .collect();
    println!("  {}", formatted.join("  ").dimmed().bold());
}
