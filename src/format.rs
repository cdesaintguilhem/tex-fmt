//! Core methodology for formatting a file

use crate::ignore::*;
use crate::indent::*;
use crate::logging::*;
use crate::parse::*;
use crate::regexes::RE_SECTION_SHARED_LINE;
use crate::regexes::{
    CHAPTER, ENV_BEGIN, ENV_END, ITEM, SECTION, SUB_SECTION, SUB_SUB_SECTION,
};
use crate::subs::*;
use crate::verbatim::*;
use crate::wrap::*;
use crate::LINE_END;
use log::Level::{Info, Warn};
use std::iter::zip;

/// Central function to format a file
pub fn format_file(
    text: &str,
    file: &str,
    args: &Cli,
    logs: &mut Vec<Log>,
) -> String {
    record_file_log(logs, Info, file, "Formatting started.");
    let mut old_text = remove_extra_newlines(text);
    if !args.usetabs {
        old_text = remove_tabs(&old_text, args);
    }
    old_text = remove_trailing_spaces(&old_text);

    let mut state = State::new();
    let old_lines = old_text.lines();
    let mut old_lines = zip(1.., old_lines);
    let mut queue: Vec<(usize, String)> = vec![];
    let mut new_text = String::with_capacity(text.len());
    let indent_char = if args.usetabs { "\t" } else { " " };

    let mut previous_line = String::default();

    loop {
        if let Some((linum_old, mut line)) = queue.pop() {
            let pattern = Pattern::new(&line);
            let mut temp_state: State;

            (line, temp_state) = apply_indent(
                &line,
                linum_old,
                &state,
                logs,
                file,
                args,
                &pattern,
                indent_char,
            );

            if !temp_state.verbatim.visual
                && !temp_state.ignore.visual
                && pattern.contains_section
                && linum_old > 1
            {
                if let Some(captures) = RE_SECTION_SHARED_LINE.captures(&line) {
                    // This regex captures sectioning commands that have text before them with named capture groups for
                    // the text before and after the command.

                    // Add the text before the command to the new text with a trailing line ending
                    new_text.push_str(&captures["prev"]);
                    new_text.push_str(LINE_END);
                    temp_state.linum_new += 1;

                    // Add an an empty line before the sectioning command
                    new_text.push_str(LINE_END);
                    temp_state.linum_new += 1;

                    // Add the sectioning command with a trailing line ending
                    new_text.push_str(&captures["command"]);
                    new_text.push_str(LINE_END);
                    temp_state.linum_new += 1;

                    // If there was trailing text after the sectioning command, add it as a new line to be processed
                    if !captures["next"].is_empty() {
                        queue.push((linum_old, captures["next"].to_string()));
                    }
                } else {
                    // When the regex doesn't match we are on a line that has a sectioning command with no text
                    // preceding it

                    // If the previous line is not empty, insert a new empty line before the sectioning command
                    if !previous_line.is_empty() {
                        new_text.push_str(LINE_END);
                        temp_state.linum_new += 1;
                    }

                    // Add the line with the sectioning command to the new text
                    new_text.push_str(&line);
                    new_text.push_str(LINE_END);
                    temp_state.linum_new += 1;
                }
                state = temp_state;
            } else if needs_env_new_line(&line, &temp_state, &pattern) {
                let env_lines =
                    put_env_new_line(&line, &temp_state, file, args, logs);
                if env_lines.is_some() {
                    queue.push((linum_old, env_lines.clone().unwrap().1));
                    queue.push((linum_old, env_lines.clone().unwrap().0));
                } else {
                    state = temp_state;
                    new_text.push_str(&line);
                    new_text.push_str(LINE_END);
                    state.linum_new += 1;
                };
            } else if needs_wrap(&line, &temp_state, args) {
                let wrapped_lines =
                    apply_wrap(&line, &temp_state, file, args, logs);
                if wrapped_lines.is_some() {
                    queue.push((linum_old, wrapped_lines.clone().unwrap().1));
                    queue.push((linum_old, wrapped_lines.clone().unwrap().0));
                } else {
                    state = temp_state;
                    new_text.push_str(&line);
                    new_text.push_str(LINE_END);
                    state.linum_new += 1;
                };
            } else {
                state = temp_state;
                new_text.push_str(&line);
                new_text.push_str(LINE_END);
                state.linum_new += 1;
            }

            // Save the line that was just processed for reference for the next one
            previous_line = line;
        } else if let Some((linum_old, line)) = old_lines.next() {
            queue.push((linum_old, line.to_string()));
        } else {
            break;
        }
    }

    if !indents_return_to_zero(&new_text) {
        record_file_log(logs, Warn, file, "Indent does not return to zero.");
    }

    new_text = remove_trailing_spaces(&new_text);
    record_file_log(logs, Info, file, "Formatting complete.");
    new_text
}

/// Information on the current state during formatting
#[derive(Clone, Debug)]
pub struct State {
    /// Corresponding line number in the original file
    pub linum_old: usize,
    /// Corresponding line number in the formatted file
    pub linum_new: usize,
    /// Ignored status of the current line
    pub ignore: Ignore,
    /// Indentation status of the current line
    pub indent: Indent,
    /// Verbatim status of the current line
    pub verbatim: Verbatim,
}

impl State {
    /// Construct a new default state
    pub const fn new() -> Self {
        Self {
            linum_old: 1,
            linum_new: 1,
            ignore: Ignore::new(),
            indent: Indent::new(),
            verbatim: Verbatim::new(),
        }
    }
}

/// Record whether a line contains certain patterns to avoid recomputing
pub struct Pattern {
    /// Wheter a section-type command pattern is present
    pub contains_section: bool,
    /// Whether a begin environment pattern is present
    pub contains_env_begin: bool,
    /// Whether an end environment pattern is present
    pub contains_env_end: bool,
    /// Whether an item pattern is present
    pub contains_item: bool,
}

impl Pattern {
    /// Check if a string contains patterns
    pub fn new(s: &str) -> Self {
        Self {
            contains_section: {
                s.contains(CHAPTER)
                    || s.contains(SECTION)
                    || s.contains(SUB_SECTION)
                    || s.contains(SUB_SUB_SECTION)
            },
            contains_env_begin: s.contains(ENV_BEGIN),
            contains_env_end: s.contains(ENV_END),
            contains_item: s.contains(ITEM),
        }
    }
}

/// Ensure that the indentation returns to zero at the end of the file
fn indents_return_to_zero(text: &str) -> bool {
    !text.lines().last().unwrap_or_default().starts_with(' ')
}
