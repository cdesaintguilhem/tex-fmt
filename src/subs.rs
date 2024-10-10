//! Utilities for performing text substitutions

use crate::comments::*;
use crate::format::*;
use crate::logging::*;
use crate::regexes::*;
use crate::Cli;
use crate::LINE_END;
use log::Level::Trace;

/// Remove multiple line breaks
pub fn remove_extra_newlines(text: &str) -> String {
    let double_line_end = format!("{LINE_END}{LINE_END}");
    RE_NEWLINES.replace_all(text, double_line_end).to_string()
}

/// Replace tabs with spaces
pub fn remove_tabs(text: &str, args: &Cli) -> String {
    let replace = (0..args.tab).map(|_| " ").collect::<String>();
    text.replace('\t', &replace)
}

/// Remove trailing spaces from line endings
pub fn remove_trailing_spaces(text: &str) -> String {
    RE_TRAIL.replace_all(text, LINE_END).to_string()
}

/// Check if environment should be split onto a new line
pub fn needs_env_new_line(
    line: &str,
    state: &State,
    pattern: &Pattern,
) -> bool {
    !state.verbatim.visual
        && !state.ignore.visual
        && (pattern.contains_env_begin
            || pattern.contains_env_end
            || pattern.contains_item)
        && (RE_ENV_BEGIN_SHARED_LINE.is_match(line)
            || RE_ENV_END_SHARED_LINE.is_match(line)
            || RE_ITEM_SHARED_LINE.is_match(line))
}

/// Ensure LaTeX environments begin on new lines
pub fn put_env_new_line<'a>(
    line: &'a str,
    state: &State,
    file: &str,
    args: &Cli,
    logs: &mut Vec<Log>,
) -> (&'a str, Option<&'a str>) {
    if args.trace {
        record_line_log(
            logs,
            Trace,
            file,
            state.linum_new,
            state.linum_old,
            line,
            "Placing environment on new line.",
        );
    }

    // If there is one, find the index of the start of the comment and split the line into its comment and text parts.
    let comment_index = find_comment_index(line);

    let captures = RE_ENV_ITEM_SHARED_LINE
        .captures(line)
        .expect("This captures because the pattern says so.");

    let (line, [prev, rest, _]) = captures.extract();

    if comment_index.is_some() && captures.get(2).unwrap().start() > comment_index.unwrap() {
        (line, None)
    } else {
        (prev, Some(rest))
    }
}
