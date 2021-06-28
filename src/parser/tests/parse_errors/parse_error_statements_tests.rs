// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use crate::{
    ast::{SourceRange, Statement},
    parser::{parse, tests::lex},
    Diagnostic,
};
use pretty_assertions::*;

/// helper function to create references
fn ref_to(name: &str) -> Statement {
    Statement::Reference {
        location: SourceRange::undefined(),
        name: name.to_string(),
    }
}

/*
 * These tests deal with parsing-behavior in the expressions: ()  expressions: ()  presence of errors.
 * following scenarios will be tested:
 *  - missing semicolons at different locations
 *  - incomplete statements
 *  - incomplete statement-blocks (brackets)
 */

#[test]
fn missing_semicolon_after_call() {
    /*
     * missing ';' after buz will be reported, both calls should be
     * parsed correctly
     */
    let lexer = lex(r"
                PROGRAM foo 
                    buz()
                    foo();
                END_PROGRAM
    ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    //expected end of statement (e.g. ;), but found KeywordEndProgram at line: 1 offset: 14..25"
    //Expecting a missing semicolon message
    let expected = Diagnostic::unexpected_token_found(
        "KeywordSemicolon".into(),
        "'foo'".into(),
        SourceRange::new("", 76..79),
    );
    assert_eq!(diagnostics[0], expected);

    let pou = &compilation_unit.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        r#"[
    CallStatement {
        operator: Reference {
            name: "buz",
        },
        parameters: None,
    },
    CallStatement {
        operator: Reference {
            name: "foo",
        },
        parameters: None,
    },
]"#
    );
}

#[test]
fn missing_comma_in_call_parameters() {
    /*
     * the missing comma after b will end the expression-list so we expect a ')'
     * c will not be parsed as an expression
     */ 
    let lexer = lex(r"
                PROGRAM foo 
                    buz(a,b c);
                END_PROGRAM
    ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    let expected = Diagnostic::unexpected_token_found(
        "KeywordParensClose".into(),
        "'c'".into(),
        SourceRange::new("", 58..59),
    );
    assert_eq!(diagnostics, vec![expected]);

    let pou = &compilation_unit.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        format!(
            "{:#?}",
            vec![Statement::CallStatement {
                location: SourceRange::undefined(),
                operator: Box::new(ref_to("buz")),
                parameters: Box::new(Some(Statement::ExpressionList {
                    expressions: vec![ref_to("a"), ref_to("b"),]
                }))
            }]
        )
    );
}

#[test]
fn illegal_semicolon_in_call_parameters() {
    /*
    * _ the semicolon after b will close the call-statement
    * _ c will be its own reference with an illegal token ')'
    */
    let lexer = lex(r"
                PROGRAM foo 
                    buz(a,b; c);
                END_PROGRAM
    ");

    let (compilation_unit, _, diagnostics) = parse(lexer).unwrap();
    let expected = Diagnostic::unexpected_token_found(
        "KeywordParensClose".into(),
        "';'".into(),
        SourceRange::new("", 57..58),
    );
    assert_eq!(diagnostics[0], expected);

    let pou = &compilation_unit.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        format!(
            "{:#?}",
            vec![
                Statement::CallStatement {
                    location: SourceRange::undefined(),
                    operator: Box::new(ref_to("buz")),
                    parameters: Box::new(Some(Statement::ExpressionList {
                        expressions: vec![ref_to("a"), ref_to("b")]
                    }))
                },
                ref_to("c"),
                Statement::EmptyStatement{location: SourceRange::undefined()}, // why ???
            ]
        )
    );
}

#[test]
fn incomplete_statement_test() {
    let lexer = lex("
        PROGRAM exp 
            1 + 2 +;
            x;
        END_PROGRAM
        ");

    let (cu, _, diagnostics) = parse(lexer).unwrap();
    let pou = &cu.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        r#"[
    EmptyStatement,
    Reference {
        name: "x",
    },
]"#
    );

    assert_eq!(
        diagnostics[0],
        Diagnostic::syntax_error("Unexpected token: ';'".into(), SourceRange::new("", 41..42))
    );
}

#[test]
fn incomplete_statement_in_parantheses_recovery_test() {
    let lexer = lex("
        PROGRAM exp 
            (1 + 2 - ) + 3;
            x;
        END_PROGRAM
        ");

    let (cu, _, diagnostics) = parse(lexer).unwrap();
    let pou = &cu.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        r#"[
    BinaryExpression {
        operator: Plus,
        left: EmptyStatement,
        right: LiteralInteger {
            value: "3",
        },
    },
    Reference {
        name: "x",
    },
]"#
    );

    assert_eq!(
        diagnostics[0],
        Diagnostic::syntax_error("Unexpected token: ')'".into(), SourceRange::new("", 43..44))
    );
}

#[test]
fn mismatched_parantheses_recovery_test() {
    let lexer = lex("
        PROGRAM exp 
            (1 + 2;
            x;
        END_PROGRAM
        ");

    let (cu, _, diagnostics) = parse(lexer).unwrap();
    let pou = &cu.implementations[0];
    assert_eq!(
        format!("{:#?}", pou.statements),
        r#"[
    BinaryExpression {
        operator: Plus,
        left: LiteralInteger {
            value: "1",
        },
        right: LiteralInteger {
            value: "2",
        },
    },
    Reference {
        name: "x",
    },
]"#
    );

    assert_eq!(
        diagnostics[0],
        Diagnostic::unexpected_token_found(
            "KeywordParensClose".into(),
            "';'".into(),
            SourceRange::new("", 40..41)
        )
    );
}

#[test]
fn invalid_variable_name_error_recovery() {
    let lexer = lex("
        PROGRAM p
            VAR 
                a b: INT;
                c : INT;
            END_VAR
        END_PROGRAM
        ");

    let (cu, _, diagnostics) = parse(lexer).unwrap();
    let pou = &cu.units[0];
    assert_eq!(
        format!("{:#?}", pou.variable_blocks[0]),
        r#"VariableBlock {
    variables: [
        Variable {
            name: "a",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
        Variable {
            name: "c",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
    ],
    variable_block_type: Local,
}"#
    );

    assert_eq!(
        diagnostics[0],
        Diagnostic::unexpected_token_found(
            "Identifier".into(),
            "':', (KeywordColon)".into(),
            SourceRange::new("", 40..41)
        )
    );
}

#[test]
fn invalid_variable_data_type_error_recovery() {
    let lexer = lex("
        PROGRAM p
            VAR 
                a INT : ;
                c : INT;
            END_VAR
        END_PROGRAM
        ");

    let (cu, _, diagnostics) = parse(lexer).unwrap();
    let pou = &cu.units[0];
    assert_eq!(
        format!("{:#?}", pou.variable_blocks[0]),
        r#"VariableBlock {
    variables: [
        Variable {
            name: "c",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
    ],
    variable_block_type: Local,
}"#
    );

    assert_eq!(
        diagnostics[0],
        Diagnostic::unexpected_token_found(
            "Identifier".into(),
            "':', (KeywordColon)".into(),
            SourceRange::new("", 40..41)
        )
    );
}
