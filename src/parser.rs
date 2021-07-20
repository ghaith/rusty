// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use crate::ast::*;
use crate::lexer;
use crate::lexer::Token;
use crate::lexer::{ParseSession, Token::*};
use crate::{ast::Implementation, Diagnostic};

use self::{control_parser::parse_control_statement, expressions_parser::parse_primary_expression};

mod control_parser;
mod expressions_parser;

#[cfg(test)]
mod tests;

#[macro_export]
macro_rules! expect {
    ( $token:expr, $lexer:expr) => {
        if $lexer.token != $token {
            return Err(Diagnostic::unexpected_token_found(
                format!("{:?}", $token),
                $lexer.slice().to_string(),
                $lexer.location(),
            ));
        }
    };
}

#[macro_export]
macro_rules! consume_or_report {
    ( $token:expr, $lexer:expr) => {
        if !crate::parser::allow($token, $lexer) {
            $lexer.accept_diagnostic(Diagnostic::missing_token(
                format!("{:?}", $token),
                $lexer.location(),
            ));
        }
    };
}

/// consumes an optional token and returns true if it was consumed.
pub fn allow(token: lexer::Token, lexer: &mut ParseSession) -> bool {
    if lexer.token == token {
        lexer.advance();
        true
    } else {
        false
    }
}

///
/// returns an error for an unidientified token
///  
fn unidentified_token(lexer: &ParseSession) -> Diagnostic {
    Diagnostic::syntax_error(
        format!("Unidentified token: {t:?}", t = lexer.slice()),
        lexer.location(),
    )
}

///
/// returns an error for an unexpected token
///  
fn unexpected_token(lexer: &ParseSession) -> Diagnostic {
    Diagnostic::syntax_error(
        format!("Unexpected token: '{slice:}'", slice = lexer.slice()),
        lexer.location(),
    )
}

fn slice_and_advance(lexer: &mut ParseSession) -> String {
    let slice = lexer.slice().to_string();
    lexer.advance();
    slice
}

pub type PResult<T> = Result<T, Diagnostic>;
pub type ParsedAst = (CompilationUnit, Vec<Diagnostic>);

pub fn parse(mut lexer: ParseSession) -> PResult<ParsedAst> {
    let mut unit = CompilationUnit::default();

    let mut linkage = LinkageType::Internal;
    loop {
        match lexer.token {
            PropertyExternal => {
                linkage = LinkageType::External;
                lexer.advance();
                //Don't reset linkage
                continue;
            }
            KeywordVarGlobal => unit
                .global_vars
                .push(parse_variable_block(&mut lexer, VariableBlockType::Global)),
            KeywordProgram => {
                if let Some((pou, implementation)) =
                    parse_pou(&mut lexer, PouType::Program, linkage, KeywordEndProgram)
                {
                    unit.units.push(pou);
                    unit.implementations.push(implementation);
                }
            }
            KeywordFunction => {
                if let Some((pou, implementation)) =
                    parse_pou(&mut lexer, PouType::Function, linkage, KeywordEndFunction)
                {
                    unit.units.push(pou);
                    unit.implementations.push(implementation);
                }
            }
            KeywordFunctionBlock => {
                if let Some((pou, implementation)) = parse_pou(
                    &mut lexer,
                    PouType::FunctionBlock,
                    linkage,
                    KeywordEndFunctionBlock,
                ) {
                    unit.units.push(pou);
                    unit.implementations.push(implementation);
                }
            }
            KeywordAction => {
                if let Some(implementation) = parse_action(&mut lexer, linkage, None) {
                    unit.implementations.push(implementation);
                }
            }
            KeywordActions => {
                let mut actions = parse_actions(&mut lexer, linkage)?;
                unit.implementations.append(&mut actions);
            }
            KeywordType => {
                if let Some(unit_type) = parse_type(&mut lexer) {
                    unit.types.push(unit_type);
                }
            }
            KeywordEndActions | End => return Ok((unit, lexer.diagnostics)),
            Error => return Err(unidentified_token(&lexer)),
            _ => return Err(unexpected_token(&lexer)),
        };
        linkage = LinkageType::Internal;
    }
    //the match in the loop will always return
}

fn parse_actions(
    mut lexer: &mut ParseSession,
    linkage: LinkageType,
) -> Result<Vec<Implementation>, Diagnostic> {
    lexer.advance(); //Consume ACTIONS
    expect!(Identifier, lexer);
    let container = slice_and_advance(lexer);
    let mut result = vec![];

    //Go through each action
    while lexer.token != KeywordEndActions && !is_end_of_stream(&lexer.token) {
        match lexer.token {
            KeywordAction => {
                if let Some(implementation) = parse_action(&mut lexer, linkage, Some(&container)) {
                    result.push(implementation);
                }
            }
            Error => return Err(unidentified_token(&lexer)),
            _ => return Err(unexpected_token(&lexer)),
        }
    }
    lexer.advance(); //Consume end actions

    Ok(result)
}

///
/// parse a pou
/// # Arguments
///
/// * `lexer`       - the lexer
/// * `pou_type`    - the type of the pou currently parsed
/// * `expected_end_token` - the token that ends this pou
///
fn parse_pou(
    lexer: &mut ParseSession,
    pou_type: PouType,
    linkage: LinkageType,
    expected_end_token: lexer::Token,
) -> Option<(Pou, Implementation)> {
    let start = lexer.range().start;
    lexer.advance(); //Consume ProgramKeyword
    let closing_tokens = vec![
        expected_end_token.clone(),
        KeywordEndAction,
        KeywordEndProgram,
        KeywordEndFunction,
        KeywordEndFunctionBlock,
    ];
    let pou = parse_any_in_region(lexer, closing_tokens.clone(), |lexer| {
        //Parse pou name
        let name = if lexer.token == Identifier {
            slice_and_advance(lexer)
        } else {
            //missing pou name
            lexer.accept_diagnostic(Diagnostic::unexpected_token_found(
                "Identifier".to_string(),
                lexer.slice().to_string(),
                SourceRange::new(lexer.range()),
            ));
            "".to_string()
        };

        //optional return type
        let start_return_type = lexer.range().start;
        let return_type = if allow(KeywordColon, lexer) {
            if lexer.token == Identifier || lexer.token == KeywordString {
                if pou_type != PouType::Function {
                    lexer.accept_diagnostic(Diagnostic::return_type_not_supported(
                        &pou_type,
                        SourceRange::new(start_return_type..lexer.range().end),
                    ));
                }
                let referenced_type = slice_and_advance(lexer);
                Some(DataTypeDeclaration::DataTypeReference {
                    referenced_type,
                    location: lexer.last_range.clone().into(),
                })
            } else {
                //missing return type
                lexer.accept_diagnostic(Diagnostic::unexpected_token_found(
                    "Datatype".to_string(),
                    lexer.slice().to_string(),
                    SourceRange::new(lexer.range()),
                ));
                None
            }
        } else {
            None
        };

        //Parse variable declarations
        let mut variable_blocks = vec![];
        while lexer.token == KeywordVar
            || lexer.token == KeywordVarInput
            || lexer.token == KeywordVarOutput
            || lexer.token == KeywordVarInOut
        {
            variable_blocks.push(parse_variable_block(
                lexer,
                parse_variable_block_type(&lexer.token),
            ));
        }

        let implementation = parse_implementation(lexer, linkage, pou_type, &name, &name);

        let pou = Pou {
            name,
            pou_type,
            variable_blocks,
            return_type,
            location: SourceRange::new(start..lexer.range().end),
        };

        Ok((pou, implementation))
    });

    //check if we ended on the right end-keyword
    if closing_tokens.contains(&lexer.last_token) && lexer.last_token != expected_end_token {
        lexer.accept_diagnostic(Diagnostic::unexpected_token_found(
            format!("{:?}", expected_end_token),
            lexer.slice_region(lexer.last_range.clone()).into(),
            SourceRange::new(lexer.last_range.clone()),
        ));
    }
    pou
}

fn parse_implementation(
    lexer: &mut ParseSession,
    linkage: LinkageType,
    pou_type: PouType,
    call_name: &str,
    type_name: &str,
) -> Implementation {
    let start = lexer.range().start;
    let statements = parse_body_standalone(lexer).unwrap_or_default();
    Implementation {
        name: call_name.into(),
        type_name: type_name.into(),
        linkage,
        pou_type,
        statements,
        location: SourceRange::new(start..lexer.range().end),
    }
}

fn parse_action(
    lexer: &mut ParseSession,
    linkage: LinkageType,
    container: Option<&str>,
) -> Option<Implementation> {
    lexer.advance(); //Consume the Action keyword
    let closing_tokens = vec![
        KeywordEndAction,
        KeywordEndProgram,
        KeywordEndFunction,
        KeywordEndFunctionBlock,
    ];

    parse_any_in_region(lexer, closing_tokens.clone(), |lexer| {
        let name_or_container = slice_and_advance(lexer);
        let (container, name) = if let Some(container) = container {
            (container.into(), name_or_container)
        } else {
            expect!(KeywordDot, lexer);
            lexer.advance();
            expect!(Identifier, lexer);
            let name = slice_and_advance(lexer);
            (name_or_container, name)
        };
        let call_name = format!("{}.{}", &container, &name);

        let implementation =
            parse_implementation(lexer, linkage, PouType::Action, &call_name, &container);
        //lets see if we ended on the right END_ keyword
        if closing_tokens.contains(&lexer.last_token) && lexer.last_token != KeywordEndAction {
            lexer.accept_diagnostic(Diagnostic::unexpected_token_found(
                format!("{:?}", KeywordEndAction),
                lexer.slice().into(),
                lexer.location(),
            ))
        }
        Ok(implementation)
    })
}

// TYPE ... END_TYPE
fn parse_type(lexer: &mut ParseSession) -> Option<UserTypeDeclaration> {
    lexer.advance(); // consume the TYPE
    let name = slice_and_advance(lexer);
    consume_or_report!(KeywordColon, lexer);

    let result = parse_full_data_type_definition(lexer, Some(name));

    if let Some((DataTypeDeclaration::DataTypeDefinition { data_type, .. }, initializer)) = result {
        consume_or_report!(KeywordEndType, lexer);
        Some(UserTypeDeclaration {
            data_type,
            initializer,
        })
    // } else {
    //     //What do we do if we want to continue parsing :(
    //     Err(Diagnostic::unexpected_token_found(
    //         "struct, enum or subrange".into(),
    //         lexer.slice().into(),
    //         lexer.location(),
    //     ))
    } else {
        None
    }
}

type DataTypeWithInitializer = (DataTypeDeclaration, Option<Statement>);

fn parse_full_data_type_definition(
    lexer: &mut ParseSession,
    name: Option<String>,
) -> Option<DataTypeWithInitializer> {
    let end_keyword = if lexer.token == KeywordStruct {
        KeywordEndStruct
    } else {
        KeywordSemicolon
    };
    parse_any_in_region(lexer, vec![end_keyword], |lexer| {
        if allow(KeywordDotDotDot, lexer) {
            Ok((
                DataTypeDeclaration::DataTypeDefinition {
                    data_type: DataType::VarArgs {
                        referenced_type: None,
                    },
                    location: lexer.last_range.clone().into(),
                },
                None,
            ))
        } else {
            parse_data_type_definition(lexer, name).map(|(type_def, initializer)| {
                if allow(KeywordDotDotDot, lexer) {
                    (
                        DataTypeDeclaration::DataTypeDefinition {
                            data_type: DataType::VarArgs {
                                referenced_type: Some(Box::new(type_def)),
                            },
                            location: lexer.last_range.clone().into(),
                        },
                        None,
                    )
                } else {
                    (type_def, initializer)
                }
            })
        }
    })
}

// TYPE xxx : 'STRUCT' | '(' | IDENTIFIER
fn parse_data_type_definition(
    lexer: &mut ParseSession,
    name: Option<String>,
) -> Result<DataTypeWithInitializer, Diagnostic> {
    let result = if allow(KeywordStruct, lexer) {
        //STRUCT
        let start = lexer.last_range.start;
        let mut variables = Vec::new();
        while lexer.token == Identifier {
            if let Some(variable) = parse_variable(lexer) {
                variables.push(variable);
            }
        }
        Ok((
            DataTypeDeclaration::DataTypeDefinition {
                data_type: DataType::StructType { name, variables },
                location: (start..lexer.range().end).into(),
            },
            None,
        ))
    } else if allow(KeywordArray, lexer) {
        parse_array_type_definition(lexer, name)
    } else if allow(KeywordParensOpen, lexer) {
        parse_enum_type_definition(lexer, name)
    } else if lexer.token == KeywordString || lexer.token == KeywordWideString {
        parse_string_type_definition(lexer, name)
    } else if lexer.token == Identifier {
        parse_type_reference_type_definition(lexer, name)
    } else {
        //no datatype?
        Err(Diagnostic::unexpected_token_found(
            "DataTypeDefinition".into(),
            format!("{:?}", lexer.token),
            lexer.location(),
        ))
    };

    result
}

fn parse_type_reference_type_definition(
    lexer: &mut ParseSession,
    name: Option<String>,
) -> PResult<(DataTypeDeclaration, Option<Statement>)> {
    let start = lexer.range().start;
    let referenced_type = slice_and_advance(lexer);

    let bounds = if allow(KeywordParensOpen, lexer) {
        //Subrange
        // INT (..) :=
        let bounds = parse_expression(lexer)?;
        expect!(KeywordParensClose, lexer);
        lexer.advance();
        Some(bounds)
    } else {
        None
    };
    let end = lexer.last_range.end;

    let initial_value = if allow(KeywordAssignment, lexer) {
        Some(parse_expression(lexer)?)
    } else {
        None
    };
    if name.is_some() || bounds.is_some() {
        let data_type = DataTypeDeclaration::DataTypeDefinition {
            data_type: DataType::SubRangeType {
                name,
                referenced_type,
                bounds,
            },
            location: (start..end).into(),
        };
        Ok((data_type, initial_value))
    } else {
        Ok((
            DataTypeDeclaration::DataTypeReference {
                referenced_type,
                location: (start..end).into(),
            },
            initial_value,
        ))
    }
}

fn parse_string_type_definition(
    lexer: &mut ParseSession,
    name: Option<String>,
) -> PResult<(DataTypeDeclaration, Option<Statement>)> {
    let start = lexer.range().start;
    let is_wide = lexer.token == KeywordWideString;
    lexer.advance();

    let size = allow(KeywordSquareParensOpen, lexer)
        .then(|| {
            parse_any_in_region(lexer, vec![KeywordSquareParensClose], |lexer| {
                let size_statement = parse_expression(lexer)?;
                Ok(size_statement)
            })
        })
        .flatten();
    let end = lexer.range().end;

    let initializer = if allow(KeywordAssignment, lexer) {
        Some(parse_expression(lexer)?)
    } else {
        None
    };
    Ok((
        DataTypeDeclaration::DataTypeDefinition {
            data_type: DataType::StringType {
                name,
                is_wide,
                size,
            },
            location: (start..end).into(),
        },
        initializer,
    ))
}

fn parse_enum_type_definition(
    lexer: &mut ParseSession,
    name: Option<String>,
) -> PResult<(DataTypeDeclaration, Option<Statement>)> {
    let start = lexer.last_range.start;
    let elements = parse_any_in_region(lexer, vec![KeywordParensClose], |lexer| {
        //ENUM
        let mut elements = Vec::new();
        //we expect at least one element
        expect!(Identifier, lexer);
        elements.push(slice_and_advance(lexer));
        //parse additional elements separated by ,
        while allow(KeywordComma, lexer) {
            expect!(Identifier, lexer);
            elements.push(slice_and_advance(lexer));
        }
        Ok(elements)
    })
    .unwrap_or_default();

    Ok((
        DataTypeDeclaration::DataTypeDefinition {
            data_type: DataType::EnumType { name, elements },
            location: (start..lexer.last_range.end).into(),
        },
        None,
    ))
}

fn parse_array_type_definition(
    lexer: &mut ParseSession,
    name: Option<String>,
) -> PResult<(DataTypeDeclaration, Option<Statement>)> {
    let start = lexer.last_range.start;
    let range = parse_statement_in_region(lexer, vec![KeywordOf], |lexer| {
        //ARRAY
        //expect open square
        expect!(KeywordSquareParensOpen, lexer);
        lexer.advance();
        //parse range
        let range = parse_primary_expression(lexer);
        //expect close range
        expect!(KeywordSquareParensClose, lexer);
        lexer.advance();
        range
    });
    let inner_type_defintion = parse_data_type_definition(lexer, None);
    inner_type_defintion.map(|(reference, initializer)| {
        let end = reference.get_location().get_end();
        (
            DataTypeDeclaration::DataTypeDefinition {
                data_type: DataType::ArrayType {
                    name,
                    bounds: range,
                    referenced_type: Box::new(reference),
                },
                location: (start..end).into(),
            },
            initializer,
        )
    })
}

fn is_end_of_stream(token: &lexer::Token) -> bool {
    *token == End || *token == Error
}

/// parse a body and recovers until the given `end_keywords`
fn parse_body_in_region(
    lexer: &mut ParseSession,
    end_keywords: Vec<Token>,
) -> Result<Vec<Statement>, Diagnostic> {
    let statements = parse_any_in_region(lexer, end_keywords, |lexer| parse_body_standalone(lexer))
        .unwrap_or_default();

    Ok(statements)
}

fn parse_body_standalone(lexer: &mut ParseSession) -> PResult<Vec<Statement>> {
    let mut statements = Vec::new();
    while !lexer.closes_open_region(&lexer.token) {
        statements.push(parse_control(lexer)?);
    }
    Ok(statements)
}

/**
 * parses a statement ending with a ;
 */
fn parse_statement(lexer: &mut ParseSession) -> Result<Statement, Diagnostic> {
    let result = parse_statement_in_region(lexer, vec![KeywordSemicolon, KeywordColon], |lexer| {
        parse_expression(lexer)
    });
    if lexer.last_token == KeywordColon {
        Ok(Statement::CaseCondition {
            condition: Box::new(result),
        })
    } else {
        Ok(result)
    }
}

pub fn parse_statement_in_region<F: FnOnce(&mut ParseSession) -> PResult<Statement>>(
    lexer: &mut ParseSession,
    closing_tokens: Vec<Token>,
    parse_fn: F,
) -> Statement {
    let start = lexer.range().start;
    parse_any_in_region(lexer, closing_tokens, parse_fn).unwrap_or_else(|| {
        let end = lexer.range().end;
        let location = SourceRange::new(start..end);
        //drop the originally parsed statement and replace with an empty-statement
        Statement::EmptyStatement { location }
    })
}

pub fn parse_any_in_region<T, F: FnOnce(&mut ParseSession) -> PResult<T>>(
    lexer: &mut ParseSession,
    closing_tokens: Vec<Token>,
    parse_fn: F,
) -> Option<T> {
    lexer.enter_region(closing_tokens);
    let result = parse_fn(lexer).map(Some).unwrap_or_else(|diagnostic| {
        lexer.accept_diagnostic(diagnostic);
        None
    });
    //try to recover by eating everything until we believe the parser is able to continue
    lexer.recover_until_close();
    lexer.close_region();
    //Report a diagnostic
    result
}

fn parse_expression(lexer: &mut ParseSession) -> Result<Statement, Diagnostic> {
    parse_primary_expression(lexer)
}

fn parse_reference(lexer: &mut ParseSession) -> Result<Statement, Diagnostic> {
    expressions_parser::parse_qualified_reference(lexer)
}

fn parse_control(lexer: &mut ParseSession) -> Result<Statement, Diagnostic> {
    parse_control_statement(lexer)
}

fn parse_variable_block_type(block_type: &Token) -> VariableBlockType {
    match block_type {
        KeywordVar => VariableBlockType::Local,
        KeywordVarInput => VariableBlockType::Input,
        KeywordVarOutput => VariableBlockType::Output,
        KeywordVarGlobal => VariableBlockType::Global,
        KeywordVarInOut => VariableBlockType::InOut,
        _ => VariableBlockType::Local,
    }
}

fn parse_variable_block(
    lexer: &mut ParseSession,
    variable_block_type: VariableBlockType,
) -> VariableBlock {
    //Consume the type keyword
    lexer.advance();
    let variables = parse_any_in_region(lexer, vec![KeywordEndVar], |lexer| {
        let mut variables = vec![];
        while lexer.token == Identifier {
            if let Some(variable) = parse_variable(lexer) {
                variables.push(variable);
            }
        }
        Ok(variables)
    })
    .unwrap_or_default();
    VariableBlock {
        variables,
        variable_block_type,
    }
}

fn parse_variable(lexer: &mut ParseSession) -> Option<Variable> {
    let variable_location = lexer.location();
    let name = slice_and_advance(lexer);

    //parse or recover until the colon
    if !allow(KeywordColon, lexer) {
        lexer.accept_diagnostic(Diagnostic::missing_token(
            format!("{:?}", KeywordColon),
            lexer.location(),
        ));
    }

    parse_full_data_type_definition(lexer, None).map(|(data_type, initializer)| Variable {
        name,
        data_type,
        location: variable_location,
        initializer,
    })
}
