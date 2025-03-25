#[derive(Debug)]
pub enum HtnErr {
    Condition {
        syntax: String,
        details: String,
    },
    Effect {
        syntax: String,
        details: String,
    },
    Float {
        syntax: String,
        details: String,
    },
    Int {
        syntax: String,
        details: String,
    },
    Enum {
        syntax: String,
        details: String,
    },
    Bool {
        syntax: String,
        details: String,
    },
    Operator {
        name: String,
        params: Vec<String>,
        details: String,
    },
    Schema {
        details: String,
    },
    ParserError {
        details: String,
    },
}

impl std::fmt::Display for HtnErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HtnErr::Condition { syntax, details } => {
                write!(f, "Invalid condition in statement `{syntax}`: {details}")
            }
            HtnErr::Effect { syntax, details } => {
                write!(f, "Invalid effect in statement `{syntax}`: {details}")
            }
            HtnErr::Float { syntax, details } => {
                write!(f, "Invalid float in statement `{syntax}`: {details}")
            }
            HtnErr::Int { syntax, details } => {
                write!(f, "Invalid integer in statement `{syntax}`: {details}")
            }
            HtnErr::Enum { syntax, details } => {
                write!(f, "Invalid enum in statement `{syntax}`: {details}")
            }
            HtnErr::Bool { syntax, details } => {
                write!(f, "Invalid boolean in statement `{syntax}`: {details}")
            }
            HtnErr::Operator {
                name,
                params,
                details,
            } => {
                write!(f, "Invalid operator `{name}` ({params:?}): {details}")
            }
            HtnErr::Schema { details } => {
                write!(f, "Schema error: {details}")
            }
            HtnErr::ParserError { details } => {
                write!(f, "HTN parsing error: {details}")
            }
        }
    }
}

impl std::error::Error for HtnErr {}
