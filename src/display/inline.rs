//! Inline, or "unified" diff display.

use crate::{
    constants::Side,
    display::context::{calculate_after_context, calculate_before_context, opposite_positions},
    display::hunks::Hunk,
    display::style::{self, apply_colors, apply_line_number_color},
    lines::{format_line_num, split_on_newlines, MaxLine},
    options::DisplayOptions,
    parse::{guess_language::Language, syntax::MatchedPos},
};

pub fn print(
    lhs_src: &str,
    rhs_src: &str,
    display_options: &DisplayOptions,
    lhs_positions: &[MatchedPos],
    rhs_positions: &[MatchedPos],
    hunks: &[Hunk],
    lhs_display_path: &str,
    rhs_display_path: &str,
    lang_name: &str,
    language: Option<Language>,
) {
    let (lhs_colored_lines, rhs_colored_lines) = if display_options.use_color {
        (
            apply_colors(
                lhs_src,
                Side::Left,
                display_options.syntax_highlight,
                language,
                display_options.background_color,
                lhs_positions,
            ),
            apply_colors(
                rhs_src,
                Side::Right,
                display_options.syntax_highlight,
                language,
                display_options.background_color,
                rhs_positions,
            ),
        )
    } else {
        (
            split_on_newlines(lhs_src)
                .iter()
                .map(|s| format!("{}\n", s))
                .collect(),
            split_on_newlines(rhs_src)
                .iter()
                .map(|s| format!("{}\n", s))
                .collect(),
        )
    };

    let lhs_colored_lines: Vec<_> = lhs_colored_lines
        .into_iter()
        .map(|line| style::replace_tabs(&line, display_options.tab_width))
        .collect();
    let rhs_colored_lines: Vec<_> = rhs_colored_lines
        .into_iter()
        .map(|line| style::replace_tabs(&line, display_options.tab_width))
        .collect();

    let opposite_to_lhs = opposite_positions(lhs_positions);
    let opposite_to_rhs = opposite_positions(rhs_positions);

    for (i, hunk) in hunks.iter().enumerate() {
        println!(
            "{}",
            style::header(
                lhs_display_path,
                rhs_display_path,
                i + 1,
                hunks.len(),
                lang_name,
                display_options
            )
        );

        let hunk_lines = hunk.lines.clone();

        let before_lines =
            calculate_before_context(&hunk_lines, &opposite_to_lhs, &opposite_to_rhs, 3);
        let after_lines = calculate_after_context(
            &[&before_lines[..], &hunk_lines[..]].concat(),
            &opposite_to_lhs,
            &opposite_to_rhs,
            // TODO: repeatedly calculating the maximum is wasteful.
            lhs_src.max_line(),
            rhs_src.max_line(),
            display_options.num_context_lines as usize,
        );

        for (lhs_line, _) in before_lines {
            let mut before_lines_str: String = "0#".to_owned();
            if let Some(lhs_line) = lhs_line {
                before_lines_str.push_str(&format_line_num(lhs_line));
                before_lines_str.push_str(&("#".to_owned()));
                print!(
                    "{}   {}",
                    apply_line_number_color(
                        &before_lines_str,
                        false,
                        Side::Left,
                        display_options,
                    ),
                    lhs_colored_lines[lhs_line.as_usize()]
                );
            }
        }

        for (lhs_line, _) in &hunk_lines {
            let mut hunk_lines_str: String = "-#".to_owned();
            if let Some(lhs_line) = lhs_line {
                hunk_lines_str.push_str(&format_line_num(*lhs_line));
                hunk_lines_str.push_str(&("#".to_owned()));
                print!(
                    "{}   {}",
                    apply_line_number_color(
                        &hunk_lines_str,
                        true,
                        Side::Left,
                        display_options,
                    ),
                    lhs_colored_lines[lhs_line.as_usize()]
                );
            }
        }
        for (_, rhs_line) in &hunk_lines {
            let mut hunk_lines_str: String = "+#".to_owned();
            if let Some(rhs_line) = rhs_line {
                hunk_lines_str.push_str(&format_line_num(*rhs_line));
                hunk_lines_str.push_str(&("#".to_owned()));
                print!(
                    "   {}{}",
                    apply_line_number_color(
                        &hunk_lines_str,
                        true,
                        Side::Right,
                        display_options,
                    ),
                    rhs_colored_lines[rhs_line.as_usize()]
                );
            }
        }

        for (_, rhs_line) in &after_lines {
            let mut after_lines_str: String = "0#".to_owned();
            if let Some(rhs_line) = rhs_line {
                after_lines_str.push_str(&format_line_num(*rhs_line));
                after_lines_str.push_str(&("#".to_owned()));
                print!(
                    "   {}{}",
                    apply_line_number_color(
                        &after_lines_str,
                        false,
                        Side::Right,
                        display_options,
                    ),
                    rhs_colored_lines[rhs_line.as_usize()]
                );
            }
        }
        println!();
    }
}
