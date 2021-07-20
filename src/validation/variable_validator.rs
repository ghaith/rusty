use crate::ast::{SourceRange, VariableBlock};

use super::DiagnosticAcceptor;

pub struct VariableValidator {}

impl VariableValidator {
    pub fn new() -> VariableValidator {
        VariableValidator {}
    }

    pub fn validate_variable_block(&self, block: &VariableBlock, da: &mut dyn DiagnosticAcceptor) {
        if block.variables.is_empty() {
            da.warning("Empty Variable block", SourceRange::undefined()); // todo range
        }
    }
}
