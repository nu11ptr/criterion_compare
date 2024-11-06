use flexstr::{flex_fmt, FlexStr, IntoFlex, ToCase, ToFlexStr};
use indexmap::IndexMap;

use crate::{ColumnInfo, Comparison, Formatter, TimeUnit};

const CT_URL: &str = "https://github.com/nu11ptr/criterion-table";

// *** NOTE: These are in _bytes_, not _chars_ - since ASCII right now this is ok ***
// Width of making a single item bold
const FIRST_COL_EXTRA_WIDTH: usize = "**``**".len();
// Width of a single item in bold (italics is less) + one item in back ticks + one item in parens + one space
// NOTE: Added one more "X" because we added unicode check, x, and rocket (uses only 1 per cell) that won't be 1 byte each
const USED_EXTRA_WIDTH: usize = "() ``****XX".len();

// *** GFM Formatter ***

/// This formatter outputs Github Flavored Markdown
pub struct GFMFormatter;

impl GFMFormatter {
    fn pad(buffer: &mut String, ch: char, max_width: usize, written: usize) {
        // Pad the rest of the column (inclusive to handle trailing space)
        let remaining = max_width - written;

        for _ in 0..=remaining {
            buffer.push(ch);
        }
    }

    #[inline]
    fn encode_link(s: &str) -> FlexStr {
        s.replace(' ', "-").into_flex().to_lower()
    }

    fn write_toc_entry(buffer: &mut String, entry: &str, indent: bool) {
        if indent {
            buffer.push_str("    ");
        }
        buffer.push_str("- [");
        buffer.push_str(entry);
        buffer.push_str("](#");
        buffer.push_str(&Self::encode_link(entry));
        buffer.push_str(")\n");
    }
}

impl Formatter for GFMFormatter {
    fn start(
        &mut self,
        buffer: &mut String,
        top_comments: &IndexMap<FlexStr, FlexStr>,
        tables: &[&FlexStr],
    ) {
        buffer.push_str("# Benchmarks\n\n");
        buffer.push_str("## Table of Contents\n\n");

        // Write each ToC entry in comments
        for section_entry in top_comments.keys() {
            Self::write_toc_entry(buffer, section_entry, false);
        }

        Self::write_toc_entry(buffer, "Benchmark Results", false);

        // Write each Benchmark ToC entry
        for &table_entry in tables {
            Self::write_toc_entry(buffer, table_entry, true);
        }

        buffer.push('\n');

        // Write out all the comment sections and comments
        for (header, comment) in top_comments {
            buffer.push_str("## ");
            buffer.push_str(header);
            buffer.push_str("\n\n");
            buffer.push_str(comment);
            buffer.push('\n');
        }

        buffer.push_str("## Benchmark Results\n\n");
    }

    fn end(&mut self, buffer: &mut String) {
        buffer.push_str("---\n");
        buffer.push_str("Made with [criterion-table](");
        buffer.push_str(CT_URL);
        buffer.push_str(")\n");
    }

    fn start_table(
        &mut self,
        buffer: &mut String,
        name: &FlexStr,
        comment: Option<&FlexStr>,
        columns: &[ColumnInfo],
    ) {
        // *** Title ***

        buffer.push_str("### ");
        buffer.push_str(name);
        buffer.push_str("\n\n");

        if let Some(comments) = comment {
            buffer.push_str(comments);
            buffer.push('\n');
        }

        // *** Header Row ***

        buffer.push_str("| ");
        // Safety: Any slicing up to index 1 is always safe - guaranteed to have at least one column
        let first_col_max_width = columns[0].max_width + FIRST_COL_EXTRA_WIDTH;
        Self::pad(buffer, ' ', first_col_max_width, 0);

        // Safety: Any slicing up to index 1 is always safe - guaranteed to have at least one column
        for column in &columns[1..] {
            let max_width = column.max_width + USED_EXTRA_WIDTH;

            buffer.push_str("| `");
            buffer.push_str(&column.name);
            buffer.push('`');
            Self::pad(buffer, ' ', max_width, column.name.chars().count() + 2);
        }

        buffer.push_str(" |\n");

        // *** Deliminator Row ***

        // Right now, everything is left justified
        buffer.push_str("|:");
        Self::pad(buffer, '-', first_col_max_width, 0);

        // Safety: Any slicing up to index 1 is always safe - guaranteed to have at least one column
        for column in &columns[1..] {
            let max_width = column.max_width + USED_EXTRA_WIDTH;

            buffer.push_str("|:");
            Self::pad(buffer, '-', max_width, 0);
        }

        buffer.push_str(" |\n");
    }

    fn end_table(&mut self, buffer: &mut String) {
        buffer.push('\n');
    }

    fn start_row(&mut self, buffer: &mut String, name: &FlexStr, max_width: usize) {
        // Regular row name
        let written = if !name.is_empty() {
            buffer.push_str("| **`");
            buffer.push_str(name);
            buffer.push_str("`**");
            name.chars().count() + FIRST_COL_EXTRA_WIDTH
            // Empty row name
        } else {
            buffer.push_str("| ");
            0
        };

        Self::pad(buffer, ' ', max_width + FIRST_COL_EXTRA_WIDTH, written);
    }

    fn end_row(&mut self, buffer: &mut String) {
        buffer.push_str(" |\n");
    }

    fn used_column(
        &mut self,
        buffer: &mut String,
        time: TimeUnit,
        compare: Comparison,
        max_width: usize,
    ) {
        let (time_str, speedup_str) = (time.to_flex_str(), compare.to_flex_str());

        // Allow 10% wiggle room to qualify
        let data = if compare >= 1.8 {
            // Positive = bold
            flex_fmt!("`{time_str}` (🚀 **{speedup_str}**)")
        // Allow 10% wiggle room to qualify
        } else if compare > 0.9 {
            // Positive = bold
            flex_fmt!("`{time_str}` (✅ **{speedup_str}**)")
        // Allow 10% wiggle room
        } else if compare < 0.9 {
            // Negative = italics
            flex_fmt!("`{time_str}` (❌ *{speedup_str}*)")
        } else {
            // Even = no special formatting
            flex_fmt!("`{time_str}` ({speedup_str})")
        };

        buffer.push_str("| ");
        buffer.push_str(&data);

        let max_width = max_width + USED_EXTRA_WIDTH;
        Self::pad(buffer, ' ', max_width, data.chars().count());
    }

    fn unused_column(&mut self, buffer: &mut String, max_width: usize) {
        buffer.push_str("| ");
        let data = "`N/A`";
        buffer.push_str(data);

        Self::pad(
            buffer,
            ' ',
            max_width + USED_EXTRA_WIDTH,
            data.chars().count(),
        );
    }
}
