//! Shared terminal renderers for CLI output.

use colored::{ Colorize, ColoredString };
use tabled::{ settings::Style, Table, Tabled };

use prism_core::types::trace::{ LedgerEntryDiff, StateDiff };

const BAR_WIDTH: usize = 10;

/// Row representation for StateDiff table
#[derive(Tabled)]
struct StateDiffRow {
    #[tabled(rename = "Key")]
    key: String,
    #[tabled(rename = "Before")]
    before: String,
    #[tabled(rename = "After")]
    after: String,
}

/// Render a StateDiff as a formatted table with highlighting
pub fn render_state_diff_table(state_diff: &StateDiff) -> String {
    let rows: Vec<StateDiffRow> = state_diff.entries
        .iter()
        .map(|entry| {
            let before = match &entry.before {
                Some(value) => {
                    if
                        matches!(
                            entry.change_type,
                            prism_core::types::trace::DiffChangeType::Deleted
                        )
                    {
                        value.red().to_string()
                    } else {
                        value.clone()
                    }
                }
                None => "-".to_string(),
            };

            let after = match &entry.after {
                Some(value) => {
                    if
                        matches!(
                            entry.change_type,
                            prism_core::types::trace::DiffChangeType::Created
                        )
                    {
                        value.green().to_string()
                    } else if
                        matches!(
                            entry.change_type,
                            prism_core::types::trace::DiffChangeType::Updated
                        )
                    {
                        value.green().to_string()
                    } else {
                        value.clone()
                    }
                }
                None => "-".to_string(),
            };

            let key = match entry.change_type {
                prism_core::types::trace::DiffChangeType::Created =>
                    format!("+ {}", entry.key).green().to_string(),
                prism_core::types::trace::DiffChangeType::Deleted =>
                    format!("- {}", entry.key).red().to_string(),
                prism_core::types::trace::DiffChangeType::Updated =>
                    format!("~ {}", entry.key).yellow().to_string(),
                prism_core::types::trace::DiffChangeType::Unchanged => entry.key.clone(),
            };

            StateDiffRow { key, before, after }
        })
        .collect();

    Table::new(&rows).with(Style::modern()).to_string()
}

/// Render a single LedgerEntryDiff for detailed view
pub fn render_ledger_entry_diff(entry: &LedgerEntryDiff) -> String {
    let change_symbol = match entry.change_type {
        prism_core::types::trace::DiffChangeType::Created => "+".green().to_string(),
        prism_core::types::trace::DiffChangeType::Deleted => "-".red().to_string(),
        prism_core::types::trace::DiffChangeType::Updated => "~".yellow().to_string(),
        prism_core::types::trace::DiffChangeType::Unchanged => " ".dimmed().to_string(),
    };

    let before_value = entry.before.as_deref().unwrap_or("-");
    let after_value = entry.after.as_deref().unwrap_or("-");

    format!(
        "{} {}\n  Before: {}\n  After:  {}",
        change_symbol,
        entry.key,
        before_value.red(),
        after_value.green()
    )
}

/// Renders a colored budget utilization bar for Soroban resource usage.
pub struct BudgetBar {
    label: &'static str,
    used: u64,
    limit: u64,
}

impl BudgetBar {
    pub fn new(label: &'static str, used: u64, limit: u64) -> Self {
        Self { label, used, limit }
    }

    pub fn render(&self) -> String {
        if self.limit == 0 {
            return format!("{}: [n/a] 0%", self.label);
        }

        let percent = self.percent();
        let filled = ((percent as usize) * BAR_WIDTH + 50) / 100;
        let filled = filled.min(BAR_WIDTH);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(BAR_WIDTH.saturating_sub(filled)));

        format!(
            "{}: [{}] {}% ({}/{})",
            self.label,
            self.colorize(bar),
            percent,
            self.used,
            self.limit
        )
    }

    fn percent(&self) -> u64 {
        if self.limit == 0 {
            return 0;
        }

        (self.used.saturating_mul(100) / self.limit).min(100)
    }

    fn colorize(&self, bar: String) -> ColoredString {
        match self.percent() {
            0..=69 => bar.green(),
            70..=89 => bar.yellow(),
            _ => bar.red(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BudgetBar;

    #[test]
    fn renders_expected_percentage() {
        let rendered = BudgetBar::new("CPU", 60, 100).render();

        assert!(rendered.contains("CPU:"));
        assert!(rendered.contains("60%"));
        assert!(rendered.contains("██████"));
    }

    #[test]
    fn clamps_over_limit_usage_to_full_bar() {
        let rendered = BudgetBar::new("RAM", 150, 100).render();

        assert!(rendered.contains("100%"));
        assert!(rendered.contains("██████████"));
    }

    #[test]
    fn renders_na_for_missing_limit() {
        let rendered = BudgetBar::new("CPU", 0, 0).render();

        assert_eq!(rendered, "CPU: [n/a] 0%");
    }
}
