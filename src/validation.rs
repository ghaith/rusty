use crate::{
    ast::{CompilationUnit, DataTypeDeclaration, Pou, SourceRange, Variable, VariableBlock},
    index::Index,
    Diagnostic, Severity,
};

use self::{pou_validator::PouValidator, variable_validator::VariableValidator};

mod pou_validator;
mod variable_validator;

pub trait DiagnosticAcceptor {
    fn unrseolved_reference(&mut self, reference: &str, location: SourceRange);
    fn error(&mut self, msg: &str, location: SourceRange);
    fn warning(&mut self, msg: &str, location: SourceRange);
}

pub struct Validator<'i> {
    pub diagnostic: Vec<Diagnostic>,
    index: &'i Index,

    pou_validator: PouValidator,
    variable_validator: VariableValidator,
}

impl DiagnosticAcceptor for Vec<Diagnostic> {
    fn unrseolved_reference(&mut self, reference: &str, location: SourceRange) {
        self.push(Diagnostic::SemanticError {
            message: format!("Could not resolve reference to '{:}", reference),
            range: location,
            severity: Severity::Error,
        });
    }

    fn error(&mut self, msg: &str, location: SourceRange) {
        self.push(Diagnostic::SemanticError {
            message: msg.into(),
            range: location,
            severity: Severity::Error,
        });
    }

    fn warning(&mut self, msg: &str, location: SourceRange) {
        self.push(Diagnostic::SemanticError {
            message: msg.into(),
            range: location,
            severity: Severity::Warning,
        });
    }
}

impl<'i> Validator<'i> {
    pub fn new(idx: &'i Index) -> Validator {
        Validator {
            diagnostic: Vec::new(),
            index: idx,
            pou_validator: PouValidator::new(),
            variable_validator: VariableValidator::new(),
        }
    }

    pub fn visit_unit(&mut self, unit: &CompilationUnit) {
        for pou in &unit.units {
            self.visit_pou(pou);
        }
    }

    pub fn visit_pou(&mut self, pou: &Pou) {
        self.pou_validator.validate_pou(pou, &mut self.diagnostic);

        for block in &pou.variable_blocks {
            self.visit_variable_container(block);
        }
    }

    pub fn visit_variable_container(&mut self, container: &VariableBlock) {
        self.variable_validator
            .validate_variable_block(container, &mut self.diagnostic);

        for variable in &container.variables {
            self.visit_variable(variable);
        }
    }

    pub fn visit_variable(&mut self, variable: &Variable) {
        self.visit_data_type_declaration(&variable.data_type);
    }

    pub fn visit_data_type_declaration(&mut self, declaration: &DataTypeDeclaration) {
        match declaration {
            DataTypeDeclaration::DataTypeReference {
                referenced_type,
                location,
            } => {
                //example validation
                if self.index.find_type(referenced_type).is_none() {
                    self.diagnostic
                        .unrseolved_reference(referenced_type, location.clone());
                }
            }
            DataTypeDeclaration::DataTypeDefinition { .. } => todo!(),
        }
    }
}
