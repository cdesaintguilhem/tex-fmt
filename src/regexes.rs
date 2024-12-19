//! Regexes and matching utilities

use crate::LINE_END;
use lazy_static::lazy_static;
use regex::Regex;

/// Match a LaTeX \item
pub const ITEM: &str = "\\item";
/// Match a LaTeX \begin{document}
pub const DOC_BEGIN: &str = "\\begin{document}";
/// Match a LaTeX \end{document}
pub const DOC_END: &str = "\\end{document}";
/// Match a LaTeX \begin{...}
pub const ENV_BEGIN: &str = "\\begin{";
/// Match a LaTeX \end{...}
pub const ENV_END: &str = "\\end{";
/// Acceptable LaTeX file extensions
pub const EXTENSIONS: [&str; 4] = [".tex", ".bib", ".sty", ".cls"];

/// Names of LaTeX list environments
const LISTS: [&str; 5] = [
    "itemize",
    "enumerate",
    "description",
    "inlineroman",
    "inventory",
];

/// Names of LaTeX verbatim environments
const VERBATIMS: [&str; 5] =
    ["verbatim", "Verbatim", "lstlisting", "minted", "comment"];

/// Names of LaTeX math environments
pub const MATH: [&str; 6] = [
    "equation",
    "equation*",
    "align",
    "align*",
    "tabular",
    "tabular*",
];

/// Regex matches for sectioning commands
const SPLITTING: [&str; 6] = [
    r"\\begin\{",
    r"\\end\{",
    r"\\item(?:$|[^a-zA-Z])",
    r"\\(?:sub){0,2}section\*?\{",
    r"\\chapter\*?\{",
    r"\\part\*?\{",
];

// Regexes
lazy_static! {
    // A static `String` which is a valid regex to match any one of the
    // [`SPLITTING_COMMANDS`].
    pub static ref SPLITTING_STRING: String = [
        "(",
        SPLITTING.join("|").as_str(),
        ")"
    ].concat();
    pub static ref RE_NEWLINES: Regex =
        Regex::new(&format!(r"{LINE_END}{LINE_END}({LINE_END})+")).unwrap();
    pub static ref RE_TRAIL: Regex =
        Regex::new(&format!(r" +{LINE_END}")).unwrap();
    pub static ref VERBATIMS_BEGIN: Vec<String> = VERBATIMS
        .iter()
        .map(|l| format!("\\begin{{{l}}}"))
        .collect();
    pub static ref VERBATIMS_END: Vec<String> =
        VERBATIMS.iter().map(|l| format!("\\end{{{l}}}")).collect();
    pub static ref LISTS_BEGIN: Vec<String> =
        LISTS.iter().map(|l| format!("\\begin{{{l}}}")).collect();
    pub static ref LISTS_END: Vec<String> =
        LISTS.iter().map(|l| format!("\\end{{{l}}}")).collect();
    // Regex that matches splitting commands
    pub static ref RE_SPLITTING: Regex = Regex::new(
        SPLITTING_STRING.as_str()
    )
    .unwrap();
    // Matches splitting commands with non-whitespace characters before it.
    pub static ref RE_SPLITTING_SHARED_LINE: Regex = Regex::new(
        [r"(:?\S.*?)", "(:?", SPLITTING_STRING.as_str(), ".*)"]
        .concat().as_str()
    )
    .unwrap();
    // Matches any splitting command with non-whitespace
    // characters before it, catches the previous text in a group called
    // "prev" and captures the command itself and the remaining text
    // in a group called "env".
    pub static ref RE_SPLITTING_SHARED_LINE_CAPTURE: Regex = Regex::new(
        [r"(?P<prev>\S.*?)", "(?P<env>", SPLITTING_STRING.as_str(), ".*)"]
        .concat().as_str()
    )
    .unwrap();
    // Matches LaTeX inline or display math opening commands at the start of a
    // line, optionally preceded by whitespace
    pub static ref RE_MATH_MODE_OPEN: Regex = Regex::new(r"^\s*(\\\(|\\\[)").unwrap();
    // Matches LaTeX inline or display math closing commands at the start of a
    // line, optionally preceded by whitespace,
    pub static ref RE_MATH_MODE_CLOSE: Regex = Regex::new(r"^\s*(\\\)|\\\])").unwrap();
    // Matches a string that indicates whether a LaTeX file contains preamble content
    pub static ref RE_PREAMBLE: Regex = Regex::new(r"(\\begin\{document\}|%! tex-fmt: preamble)").unwrap();
    // Matches the `\begin{document}` pattern which ends a LaTeX preamble
    pub static ref RE_END_PREAMBLE: Regex = Regex::new(r"\\begin\{document\}").unwrap();

}

#[cfg(test)]
mod tests {
    use crate::regexes::RE_PREAMBLE;

    #[test]
    fn preamble() {
        assert!(RE_PREAMBLE.is_match(
            "Preamble text.\n\\begin{document}\nText goes here.\\end{document}"
        ));
        assert!(RE_PREAMBLE.is_match(
            "%! tex-fmt: preamble\n\\documentclass{article}\n\\usepackage{}"
        ));
    }
}
