use crate::format::*;
use crate::logging::*;
use crate::regexes::*;
use log::Level::Warn;

#[derive(Clone, Debug)]
pub struct Verbatim {
    pub actual: i8,
    pub visual: bool,
}

impl Verbatim {
    pub const fn new() -> Self {
        Self {
            actual: 0,
            visual: false,
        }
    }
}

pub fn get_verbatim(
    line: &str,
    state: &State,
    logs: &mut Vec<Log>,
    file: &str,
    warn: bool,
) -> Verbatim {
    let diff = get_verbatim_diff(line);
    let actual = state.verbatim.actual + diff;
    let visual = actual > 0 && state.verbatim.actual > 0;

    if warn && (actual < 0) {
        record_line_log(
            logs,
            Warn,
            file,
            state.linum_new,
            state.linum_old,
            line,
            "Verbatim count is negative.",
        );
    }

    Verbatim { actual, visual }
}

fn get_verbatim_diff(line: &str) -> i8 {
    if line.contains(ENV_BEGIN)
        && VERBATIMS_BEGIN.iter().any(|r| line.contains(r))
    {
        1
    } else if line.contains(ENV_END)
        && VERBATIMS_END.iter().any(|r| line.contains(r))
    {
        -1
    } else {
        0
    }
}
