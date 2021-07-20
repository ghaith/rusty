use crate::ast::{Pou, PouType};

use super::DiagnosticAcceptor;

pub struct PouValidator {}

impl PouValidator {
    pub fn new() -> PouValidator {
        PouValidator {}
    }

    pub fn validate_pou(&self, pou: &Pou, da: &mut dyn DiagnosticAcceptor) {
        if pou.pou_type == PouType::Function && pou.return_type.is_none() {
            //Function without a return type
            da.error("Function Return type missing", pou.location.clone());
        } else if pou.return_type.is_some() {
            //non-function with a return-type
        }
    }
}
