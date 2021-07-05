// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use crate::{
    ast::{PouType, SourceRange, Statement},
    lexer::Token,
    parser::{parse, tests::lex},
    Diagnostic,
};
use logos::Source;
use pretty_assertions::*;

/*
 * These tests deal with parsing-behavior of containers like POUs, VAR-containers and Actions
 * in the presence of errors.
 * following scenarios will be tested:
 *  - missing pou names, missing function's return variable, illegal return-variable declaration
 *  - incomplete variable-declarations
 *  - incomplete containers, illegal close-keywords
 */

#[test]
fn missing_pou_name() {
    let lexer = lex(r"
            PROGRAM  
            VAR END_VAR
            a;
            END_PROGRAM
            ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    //expected end of statement (e.g. ;), but found KeywordEndProgram at line: 1 offset: 14..25"
    //Expecting a missing semicolon message
    let expected = Diagnostic::unexpected_token_found(
        "Identifier".into(),
        "VAR".into(),
        SourceRange::new("", 35..38),
    );
    assert_eq!(diagnostics[0], expected);

    let pou = &compilation_unit.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements[0]),
        format!(
            "{:#?}",
            Statement::Reference {
                name: "a".into(),
                location: SourceRange::undefined()
            }
        )
    );
}

#[test]
fn missing_pou_name_2() {
    // in this case, a becomes the POU's name
    let lexer = lex(r"
            PROGRAM 
            a := 2;
            x;
            END_PROGRAM
            ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    //expected end of statement (e.g. ;), but found KeywordEndProgram at line: 1 offset: 14..25"
    //Expecting a missing semicolon message
    let expected = Diagnostic::syntax_error("Unexpected token: ':='".into(), (36..38).into());
    assert_eq!(diagnostics[0], expected);

    let pou = &compilation_unit.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements[1]),
        format!(
            "{:#?}",
            Statement::Reference {
                name: "x".into(),
                location: SourceRange::undefined()
            }
        )
    );
}

#[test]
fn illegal_end_pou_keyword() {
    let lexer = lex(r"
            PROGRAM foo
            a;
            END_FUNCTION
            PROGRAM baz
            b;
            END_PROGRAM
            ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    let expected = Diagnostic::unexpected_token_found(
        format!("{:?}", Token::KeywordEndProgram),
        "END_FUNCTION".into(),
        SourceRange::new("", 52..64),
    );
    assert_eq!(diagnostics[0], expected);

    //check if baz was parsed successfully
    let pou = &compilation_unit.implementations[1];
    assert_eq!(
        format!("{:#?}", pou.statements),
        format!(
            "{:#?}",
            vec![Statement::Reference {
                name: "b".into(),
                location: SourceRange::undefined()
            }]
        )
    );
}

#[test]
#[ignore = "Semantic validation"]
fn function_without_return_variable_declaration() {
    // GIVEN a function without a return type
    let lexer = lex(r"
        FUNCTION foo
        a;
        END_FUNCTION
        ");

    // WHEN the function is parsed
    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();

    // THEN I expect a diagnostic complaining about a missing return type
    let expected = Diagnostic::unexpected_token_found(
        "COLON".into(),
        "'a', (Identifier)".into(),
        SourceRange::new("", 76..79),
    );
    assert_eq!(diagnostics, vec![expected]);

    // AND I expect the body to be parsed successfully
    let pou = &compilation_unit.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        r#"[
            Reference {
                name: "a",
            },
            ]"#
    );
}

#[test]
fn function_with_illegal_return_variable_declaration() {
    let lexer = lex(r"
            FUNCTION foo :
            VAR END_VAR
            a;
            END_FUNCTION
            ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    //expected end of statement (e.g. ;), but found KeywordEndProgram at line: 1 offset: 14..25"
    //Expecting a missing semicolon message
    let expected = Diagnostic::unexpected_token_found(
        "Datatype".into(),
        "VAR".into(),
        SourceRange::new("", 40..43),
    );
    assert_eq!(diagnostics[0], expected);

    //check if a was parsed successfully
    let pou = &compilation_unit.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        r#"[
    Reference {
        name: "a",
    },
]"#
    );
}

#[test]
fn program_with_illegal_return_variable_declaration() {
    let lexer = lex(r"
                PROGRAM foo : INT
                VAR END_VAR
                a;
                END_PROGRAM
                ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    //expected end of statement (e.g. ;), but found KeywordEndProgram at line: 1 offset: 14..25"
    //Expecting a missing semicolon message
    let expected =
        Diagnostic::return_type_not_supported(&PouType::Program, SourceRange::new("", 29..34));
    assert_eq!(diagnostics.get(0), Some(&expected));

    //check if a was parsed successfully
    let pou = &compilation_unit.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        format!(
            "{:#?}",
            vec![Statement::Reference {
                name: "a".into(),
                location: SourceRange::undefined()
            }]
        )
    );
}

#[test]
fn pou_inside_pou_body() {
    let lexer = lex(r"
                PROGRAM foo
                    VAR END_VAR
                    PROGRAM foo2 END_PROGRAM
                    a;
                END_PROGRAM
                ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    assert_eq!(
        Diagnostic::syntax_error("Unexpected token: 'PROGRAM'".into(), (81..88).into(),),
        diagnostics[0]
    );
    assert_eq!(
        Diagnostic::unexpected_token_found(
            "Semicolon".into(),
            "'END_PROGRAM', (KeywordEndProgram)".into(),
            SourceRange::undefined(),
        ),
        diagnostics[1]
    );

    //check if a was parsed successfully
    let pou = &compilation_unit.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        r#"[
                Reference {
                    name: "foo2",
                },
                Reference {
                        name: "foo2",
                    },
            ]"#
    );
}

#[test]
fn unclosed_var_container() {
    let lexer = lex(r"
                PROGRAM foo
                    VAR a : INT;
                    VAR b : INT; END_VAR
                END_PROGRAM
                ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    assert_eq!(
        Diagnostic::unexpected_token_found(
            "END_VAR".into(),
            "'VAR', (KeywordVAR)".into(),
            SourceRange::undefined(),
        ),
        diagnostics[0]
    );
    //check if b was parsed successfully
    let var_block = &compilation_unit.units[1].variable_blocks[1];
    assert_eq!(
        format!("{:#?}", var_block.variables[0]),
        r#"Variable {
            name: "b",
            data_type: DataTypeReference {
                referenced_tye: "INT",
            },
            initializer: None,
        }
"#
    );
}