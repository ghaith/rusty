// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
//! A Structured Text LLVM Frontent
//!
//! RuSTy is an [`ST`] Compiler using LLVM
//!
//! # Features
//! ## Standard language support
//! Most of the [`IEC61131-3`] standard for ST and general programing is supported.
//! ## Native compilation
//! A (currently) single ST files into object code using LLVM.
//! A compiled object can be linked statically or dynamically linked
//!     with other programs using standard compiler linkers (ld, clang, gcc)
//! ## IR Output
//! An [`IR`] file can be generated from any given ST file in order to examin the generated LLVM IR code.
//! For a usage guide refer to the [User Documentation](../../)
//!
//! [`ST`]: https://en.wikipedia.org/wiki/Structured_text
//! [`IEC61131-3`]: https://en.wikipedia.org/wiki/IEC_61131-3
//! [`IR`]: https://llvm.org/docs/LangRef.html
use std::path::Path;

use ast::{PouType, SourceRange};
use codespan_reporting::diagnostic::{self, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Chars, Styles};
use compile_error::CompileError;
use index::Index;
use inkwell::context::Context;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use parser::ParsedAst;
use validation::Validator;

use crate::ast::CompilationUnit;
mod ast;
pub mod cli;
mod codegen;
pub mod compile_error;
pub mod index;
mod lexer;
mod parser;
mod typesystem;
mod validation;

#[macro_use]
extern crate pretty_assertions;

#[derive(PartialEq, Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Diagnostic {
    SyntaxError {
        message: String,
        range: SourceRange,
    },
    SemanticError {
        message: String,
        range: SourceRange,
        severity: Severity,
    },
}

impl Diagnostic {
    pub fn syntax_error(message: String, range: SourceRange) -> Diagnostic {
        Diagnostic::SyntaxError { message, range }
    }

    pub fn unexpected_token_found(
        expected: String,
        found: String,
        range: SourceRange,
    ) -> Diagnostic {
        Diagnostic::SyntaxError {
            message: format!(
                "Unexpected token: expected {} but found {}",
                expected, found
            ),
            range,
        }
    }

    pub fn return_type_not_supported(pou_type: &PouType, range: SourceRange) -> Diagnostic {
        Diagnostic::syntax_error(
            format!(
                "POU Type {:?} does not support a return type. Did you mean Function?",
                pou_type
            ),
            range,
        )
    }

    pub fn missing_token(epxected_token: String, range: SourceRange) -> Diagnostic {
        Diagnostic::SyntaxError {
            message: format!("Missing expected Token {}", epxected_token),
            range,
        }
    }

    pub fn get_message(&self) -> &str {
        match self {
            Diagnostic::SyntaxError { message, .. } => message.as_str(),
            Diagnostic::SemanticError { message, .. } => message.as_str(),
        }
    }

    pub fn get_location(&self) -> SourceRange {
        match self {
            Diagnostic::SyntaxError { range, .. } => range.clone(),
            Diagnostic::SemanticError { range, .. } => range.clone(),
        }
    }
}

pub type Sources<'a> = [&'a dyn SourceContainer];

/// SourceContainers offer source-code to be compiled via the load_source function.
/// Furthermore it offers a location-String used when reporting diagnostics.
pub trait SourceContainer {
    /// loads and returns the SourceEntry that contains the SourceCode and the path it was loaded from
    fn load_source(&self) -> Result<SourceCode, String>;
    /// returns the location of this source-container. Used when reporting diagnostics.
    fn get_location(&self) -> &str;
}

/// The SourceCode unit is the smallest unit of compilation that can be passed to the compiler
#[derive(Clone)]
pub struct SourceCode {
    /// the source code to be compiled
    pub source: String,
    /// the location this code was loaded from
    pub path: String,
}

impl SourceCode {
    /// casts the SourceCode into a SourceContainer
    pub fn as_source_container(&self) -> &dyn SourceContainer {
        self
    }
}

/// tests can provide a SourceCode directly
impl SourceContainer for SourceCode {
    fn load_source(&self) -> Result<SourceCode, String> {
        Ok(self.clone())
    }

    fn get_location(&self) -> &str {
        self.path.as_str()
    }
}

///
/// Compiles the given source into an object file and saves it in output
///
fn compile_to_obj(
    sources: &Sources,
    output: &str,
    reloc: RelocMode,
    triple: Option<String>,
) -> Result<(), CompileError> {
    let initialization_config = &InitializationConfig::default();
    Target::initialize_all(initialization_config);

    let triple = triple
        .map(|it| TargetTriple::create(it.as_str()))
        .or_else(|| Some(TargetMachine::get_default_triple()))
        .unwrap();
    let target = Target::from_triple(&triple).unwrap();
    let machine = target
        .create_target_machine(
            &triple,
            //TODO : Add cpu features as optionals
            "generic", //TargetMachine::get_host_cpu_name().to_string().as_str(),
            "",        //TargetMachine::get_host_cpu_features().to_string().as_str(),
            //TODO Optimisation as parameter
            inkwell::OptimizationLevel::Default,
            reloc,
            CodeModel::Default,
        )
        .unwrap();

    let c = Context::create();
    let code_generator = compile_module(&c, sources)?;
    machine
        .write_to_file(&code_generator.module, FileType::Object, Path::new(output))
        .unwrap();

    Ok(())
}

/// Compiles a given source string to a static object and saves the output.
///
/// # Arguments
///
/// * `sources` - the source to be compiled
/// * `output` - the location on disk to save the output
/// * `target` - an optional llvm target triple
///     If not provided, the machine's triple will be used.
pub fn compile_to_static_obj(
    sources: &Sources,
    output: &str,
    target: Option<String>,
) -> Result<(), CompileError> {
    compile_to_obj(sources, output, RelocMode::Default, target)
}

/// Compiles a given source string to a shared position independent object and saves the output.
///
/// # Arguments
///
/// * `sources` - the source to be compiled
/// * `output` - the location on disk to save the output
/// * `target` - an optional llvm target triple
///     If not provided, the machine's triple will be used.
pub fn compile_to_shared_pic_object(
    sources: &Sources,
    output: &str,
    target: Option<String>,
) -> Result<(), CompileError> {
    compile_to_obj(sources, output, RelocMode::PIC, target)
}

/// Compiles a given source string to a dynamic non PIC object and saves the output.
///
/// # Arguments
///
/// * `sources` - the source to be compiled
/// * `output` - the location on disk to save the output
/// * `target` - an optional llvm target triple
///     If not provided, the machine's triple will be used.
pub fn compile_to_shared_object(
    sources: &Sources,
    output: &str,
    target: Option<String>,
) -> Result<(), CompileError> {
    compile_to_obj(sources, output, RelocMode::DynamicNoPic, target)
}

///
/// Compiles the given source into a bitcode file
///
/// # Arguments
///
/// * `sources` - the source to be compiled
/// * `output` - the location on disk to save the output
pub fn compile_to_bitcode(sources: &Sources, output: &str) -> Result<(), CompileError> {
    let path = Path::new(output);
    let c = Context::create();
    let code_generator = compile_module(&c, sources)?;
    code_generator.module.write_bitcode_to_path(path);
    Ok(())
}

///
/// Compiles the given source into LLVM IR and returns it
///
/// # Arguments
///
/// * `sources` - the source to be compiled
pub fn compile_to_ir(sources: &Sources) -> Result<String, CompileError> {
    let c = Context::create();
    let code_gen = compile_module(&c, sources)?;
    Ok(code_gen.module.print_to_string().to_string())
}

///
/// Compiles the given source into a `codegen::CodeGen` using the provided context
///
/// # Arguments
///
/// * `context` - the LLVM Context to be used for the compilation
/// * `sources` - the source to be compiled
pub fn compile_module<'c>(
    context: &'c Context,
    sources: &Sources,
) -> Result<codegen::CodeGen<'c>, CompileError> {
    let mut full_index = Index::new();
    let mut unit = CompilationUnit::default();
    // let mut diagnostics : Vec<Diagnostic> = vec![];
    let mut files: SimpleFiles<String, String> = SimpleFiles::new();
    for container in sources {
        let e = container
            .load_source()
            .map_err(|err| CompileError::io_error(err, container.get_location().to_string()))?;

        let (mut parse_result, diagnostics) = parse(e.source.as_str())?;
        ast::pre_process(&mut parse_result);
        full_index.import(index::visitor::visit(&parse_result));

        let mut validator = Validator::new(&full_index);
        validator.visit_unit(&parse_result);
        let syntactic_diagnostics = diagnostics.into_iter();
        let semantic_diagnostics = validator.diagnostic.into_iter();

        let diagnostics = syntactic_diagnostics.chain(semantic_diagnostics);
        unit.import(parse_result);
        //log errors
        let file_id = files.add(e.path.clone(), e.source.clone());
        for error in diagnostics {
            let diag = diagnostic::Diagnostic::error()
                .with_message(error.get_message())
                .with_labels(vec![Label::primary(
                    file_id,
                    error.get_location().get_start()..error.get_location().get_end(),
                )]);
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config {
                display_style: term::DisplayStyle::Rich,
                tab_width: 2,
                styles: Styles::default(),
                chars: Chars::default(),
                start_context_lines: 5,
                end_context_lines: 3,
            };

            term::emit(&mut writer.lock(), &config, &files, &diag).map_err(|err| {
                CompileError::codegen_error(
                    format!("Cannot print errors {:#?}", err),
                    SourceRange::undefined(),
                )
            })?;
        }
    }

    //and finally codegen
    let code_generator = codegen::CodeGen::new(context, "main");
    code_generator.generate(unit, &full_index)?;
    Ok(code_generator)
}

fn parse(source: &str) -> Result<ParsedAst, CompileError> {
    //Start lexing
    let lexer = lexer::lex(source);
    //Parse
    //TODO : Parser should also return compile errors with sane locations
    parser::parse(lexer).map_err(|err| err.into())
}
