//! Detecting patterns within lines

use crate::regexes::*;

/// Record whether a line contains certain patterns to avoid recomputing
#[derive(Default)]
pub struct Pattern {
    /// Whether a begin environment pattern is present
    pub contains_env_begin: bool,
    /// Whether an end environment pattern is present
    pub contains_env_end: bool,
    /// Whether an item pattern is present
    pub contains_item: bool,
    /// Whether a splitting pattern is present
    pub contains_splitting: bool,
    /// Whether the current line opens a math environment
    pub opens_math_environment: bool,
    /// Whether the current line closes a math environment
    pub closes_math_environment: bool,
}

impl Pattern {
    /// Check if a string contains patterns
    pub fn new(s: &str) -> Self {
        let mut pattern = Self::default();

        // If splitting does not match, no patterns are present
        if RE_SPLITTING.is_match(s) {
            pattern.contains_env_begin = s.contains(ENV_BEGIN);
            pattern.contains_env_end = s.contains(ENV_END);
            pattern.contains_item = s.contains(ITEM);
            pattern.contains_splitting = true;
        }

        if RE_MATH_MODE_OPEN.is_match(s)
            || (pattern.contains_env_begin && Self::contains_math_env_name(s))
        {
            pattern.opens_math_environment = true;
        }

        if RE_MATH_MODE_CLOSE.is_match(s)
            || (pattern.contains_env_end && Self::contains_math_env_name(s))
        {
            pattern.closes_math_environment = true;
        }

        pattern
    }

    /// Checks whether the given string slice contains one of the environment
    /// names in [`MATH`].
    ///
    /// Assumes that the string already contains [`ENV_BEGIN`] or [`ENV_END`].
    fn contains_math_env_name(s: &str) -> bool {
        for name in MATH {
            if s.contains(name) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    #[test]
    fn new_pattern() {
        // Testing splitting patterns
        let pattern =
            Pattern::new("\\begin{enumerate} \\end{enumerate} \\item ");
        assert!(pattern.contains_env_begin);
        assert!(pattern.contains_env_end);
        assert!(pattern.contains_item);
        assert!(pattern.contains_splitting);

        // --- Testing math patterns

        // Opening macros for inline or display math must start the line (maybe
        // with whitespace) and can have math after the macro on the same line,
        // but no text before
        assert!(Pattern::new("\\( math after").opens_math_environment);
        assert!(Pattern::new("    \\( math after").opens_math_environment);
        assert!(Pattern::new("\\[ math after").opens_math_environment);
        assert!(Pattern::new("    \\[ math after").opens_math_environment);
        assert!(!Pattern::new("text before \\(").opens_math_environment);
        assert!(!Pattern::new("text before \\[").opens_math_environment);
        assert!(!Pattern::new("text before \\)").closes_math_environment);
        assert!(!Pattern::new("text before \\]").closes_math_environment);

        // Closing macros for inline or display math must start the line (maybe
        // with whitespace) and can have text after the macro on the same line,
        // but no math before
        assert!(Pattern::new("\\) text after").closes_math_environment);
        assert!(Pattern::new("    \\) text after").closes_math_environment);
        assert!(Pattern::new("\\] text after").closes_math_environment);
        assert!(Pattern::new("    \\] text after").closes_math_environment);
        assert!(!Pattern::new("math before \\)").closes_math_environment);
        assert!(!Pattern::new("math before \\)").closes_math_environment);
        assert!(!Pattern::new("math before \\]").closes_math_environment);
        assert!(!Pattern::new("math before \\]").closes_math_environment);

        // Math environment names within `\begin{}` and `\end{}` should start
        // and end math mode respectively.
        assert!(Pattern::new("\\begin{equation}").opens_math_environment);
        assert!(Pattern::new("\\end{equation}").closes_math_environment);
        assert!(Pattern::new("\\begin{equation*}").opens_math_environment);
        assert!(Pattern::new("\\end{equation*}").closes_math_environment);

        // Other environment names should not start math mode
        assert!(!Pattern::new("\\begin{document}").opens_math_environment);
        assert!(!Pattern::new("\\begin{definition}").opens_math_environment);
        assert!(!Pattern::new("\\begin{theorem}").opens_math_environment);
        assert!(!Pattern::new("\\begin{quote}").opens_math_environment);
    }
}
