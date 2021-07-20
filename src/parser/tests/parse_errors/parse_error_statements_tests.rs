// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use crate::{
    ast::{
        ConditionalBlock, DataTypeDeclaration, SourceRange, Statement, Variable, VariableBlock,
        VariableBlockType,
    },
    lexer::Token,
    parser::{
        parse,
        tests::{empty_stmt, lex, ref_to},
    },
    Diagnostic,
};
use pretty_assertions::*;

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

    let (compilation_unit, diagnostics) = parse(lexer).unwrap();
    //expected end of statement (e.g. ;), but found KeywordEndProgram at line: 1 offset: 14..25"
    //Expecting a missing semicolon message
    let expected = Diagnostic::unexpected_token_found(
        "KeywordSemicolon".into(),
        "'foo()'".into(),
        SourceRange::new(76..81),
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

    let (compilation_unit, diagnostics) = parse(lexer).unwrap();
    let expected = Diagnostic::unexpected_token_found(
        "KeywordParensClose".into(),
        "'c'".into(),
        SourceRange::new(58..59),
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

    let (compilation_unit, diagnostics) = parse(lexer).unwrap();
    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordParensClose]".into(), SourceRange::new(57..58)),
            Diagnostic::unexpected_token_found(
                "KeywordParensClose".into(),
                "';'".into(),
                SourceRange::new(57..58)
            ),
            Diagnostic::unexpected_token_found(
                "KeywordSemicolon".into(),
                "')'".into(),
                SourceRange::new(60..61)
            )
        ]
    );

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
                ref_to("c")
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

    let (cu, diagnostics) = parse(lexer).unwrap();
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
        Diagnostic::syntax_error("Unexpected token: ';'".into(), SourceRange::new(41..42))
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

    let (cu, diagnostics) = parse(lexer).unwrap();
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
        Diagnostic::syntax_error("Unexpected token: ')'".into(), SourceRange::new(43..44))
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

    let (cu, diagnostics) = parse(lexer).unwrap();
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
        Diagnostic::missing_token("[KeywordParensClose]".into(), SourceRange::new(40..41))
    );
}

#[test]
fn invalid_variable_name_error_recovery() {
    let lexer = lex("
        PROGRAM p
            VAR 
                c : INT;
                4 : INT;
            END_VAR
        END_PROGRAM
        ");

    let (cu, diagnostics) = parse(lexer).unwrap();
    let pou = &cu.units[0];
    assert_eq!(
        format!("{:#?}", pou.variable_blocks[0]),
        format!(
            "{:#?}",
            VariableBlock {
                variables: vec![Variable {
                    name: "c".into(),
                    data_type: DataTypeDeclaration::DataTypeReference {
                        referenced_type: "INT".into(),
                        location: SourceRange::undefined(),
                    },
                    initializer: None,
                    location: SourceRange::undefined(),
                },],
                variable_block_type: VariableBlockType::Local,
            }
        )
    );

    assert_eq!(
        diagnostics[0],
        Diagnostic::unexpected_token_found(
            format!("{:?}", Token::KeywordEndVar),
            "'4 : INT;'".into(),
            SourceRange::new(77..85)
        )
    );
}

#[test]
fn invalid_variable_data_type_error_recovery() {
    let lexer = lex("
        PROGRAM p
            VAR 
                a DINT : ;
                c : INT;
            END_VAR
        END_PROGRAM
        ");

    let (cu, diagnostics) = parse(lexer).unwrap();
    let pou = &cu.units[0];
    assert_eq!(
        format!("{:#?}", pou.variable_blocks[0]),
        r#"VariableBlock {
    variables: [
        Variable {
            name: "a",
            data_type: DataTypeReference {
                referenced_type: "DINT",
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
        diagnostics,
        vec![
            Diagnostic::missing_token("KeywordColon".into(), SourceRange::new(54..58)),
            Diagnostic::unexpected_token_found(
                "KeywordSemicolon".into(),
                "':'".into(),
                SourceRange::new(59..60)
            )
        ]
    );
}

#[test]
fn test_if_with_missing_semicolon_in_body() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            IF TRUE THEN
                x := x + 1
            END_IF
        END_PROGRAM
    ");
    let (_, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordSemicolon, KeywordColon]".into(), (79..85).into()),
            Diagnostic::unexpected_token_found(
                "KeywordSemicolon".into(),
                "'END_IF'".into(),
                (79..85).into()
            )
        ]
    );
}

#[test]
fn test_nested_if_with_missing_end_if() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            IF FALSE THEN
                IF TRUE THEN
                    x := y;
            END_IF
            y := x;
        END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token(
                "[KeywordEndIf, KeywordElseIf, KeywordElse]".into(),
                (145..156).into()
            ),
            Diagnostic::unexpected_token_found(
                "KeywordEndIf".into(),
                "'END_PROGRAM'".into(),
                (145..156).into()
            ),
        ]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![Statement::IfStatement {
                blocks: vec![ConditionalBlock {
                    condition: Box::new(Statement::LiteralBool {
                        value: false,
                        location: SourceRange::undefined()
                    }),
                    body: vec![
                        Statement::IfStatement {
                            blocks: vec![ConditionalBlock {
                                condition: Box::new(Statement::LiteralBool {
                                    value: true,
                                    location: SourceRange::undefined()
                                }),
                                body: vec![Statement::Assignment {
                                    left: Box::new(ref_to("x")),
                                    right: Box::new(ref_to("y"))
                                }],
                            }],
                            else_block: vec![],
                            location: SourceRange::undefined(),
                        },
                        Statement::Assignment {
                            left: Box::new(ref_to("y")),
                            right: Box::new(ref_to("x"))
                        }
                    ]
                },],
                else_block: vec![],
                location: SourceRange::undefined(),
            },]
        )
    );
}

#[test]
fn test_for_with_missing_semicolon_in_body() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            FOR x := 1 TO 2 DO
                y := x
            END_FOR
        END_PROGRAM
    ");
    let (_, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordSemicolon, KeywordColon]".into(), (81..88).into()),
            Diagnostic::unexpected_token_found(
                "KeywordSemicolon".into(),
                "'END_FOR'".into(),
                (81..88).into()
            )
        ]
    );
}

#[test]
fn test_nested_for_with_missing_end_for() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            FOR x := 1 TO 2 DO 
                FOR x := 1 TO 2 DO 
                    y := x;
            END_FOR
            x := y;
        END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordEndFor]".into(), (159..170).into()),
            Diagnostic::unexpected_token_found(
                "KeywordEndFor".into(),
                "'END_PROGRAM'".into(),
                (159..170).into()
            ),
        ]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![Statement::ForLoopStatement {
                counter: Box::new(ref_to("x")),
                start: Box::new(Statement::LiteralInteger {
                    value: "1".into(),
                    location: SourceRange::undefined()
                }),
                end: Box::new(Statement::LiteralInteger {
                    value: "2".into(),
                    location: SourceRange::undefined()
                }),
                by_step: None,
                body: vec![
                    Statement::ForLoopStatement {
                        counter: Box::new(ref_to("x")),
                        start: Box::new(Statement::LiteralInteger {
                            value: "1".into(),
                            location: SourceRange::undefined()
                        }),
                        end: Box::new(Statement::LiteralInteger {
                            value: "2".into(),
                            location: SourceRange::undefined()
                        }),

                        by_step: None,
                        body: vec![Statement::Assignment {
                            left: Box::new(ref_to("y")),
                            right: Box::new(ref_to("x"))
                        },],
                        location: SourceRange::undefined()
                    },
                    Statement::Assignment {
                        left: Box::new(ref_to("x")),
                        right: Box::new(ref_to("y"))
                    }
                ],
                location: SourceRange::undefined()
            },]
        )
    );
}

#[test]
fn test_repeat_with_missing_semicolon_in_body() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            REPEAT
                x := 3
            UNTIL x = y END_REPEAT
            y := x;     
           END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordSemicolon, KeywordColon]".into(), (69..74).into()),
            Diagnostic::unexpected_token_found(
                "KeywordSemicolon".into(),
                "'UNTIL'".into(),
                (69..74).into()
            ),
        ]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![
                Statement::RepeatLoopStatement {
                    body: vec![Statement::Assignment {
                        left: Box::new(ref_to("x")),
                        right: Box::new(Statement::LiteralInteger {
                            value: "3".into(),
                            location: SourceRange::undefined()
                        })
                    }],
                    condition: Box::new(Statement::BinaryExpression {
                        left: Box::new(ref_to("x")),
                        right: Box::new(ref_to("y")),
                        operator: crate::ast::Operator::Equal
                    }),
                    location: SourceRange::undefined(),
                },
                Statement::Assignment {
                    left: Box::new(ref_to("y")),
                    right: Box::new(ref_to("x"))
                }
            ]
        )
    );
}

#[test]
fn test_nested_repeat_with_missing_until_end_repeat() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            REPEAT
                REPEAT
                    ;
                UNTIL x = y END_REPEAT
                y := x;     
           END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordUntil, KeywordEndRepeat]".into(), (158..169).into()),
            Diagnostic::unexpected_token_found(
                "KeywordUntil".into(),
                "'END_PROGRAM'".into(),
                (158..169).into()
            ),
        ]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![Statement::RepeatLoopStatement {
                body: vec![
                    Statement::RepeatLoopStatement {
                        body: vec![empty_stmt()],
                        condition: Box::new(Statement::BinaryExpression {
                            left: Box::new(ref_to("x")),
                            right: Box::new(ref_to("y")),
                            operator: crate::ast::Operator::Equal
                        }),
                        location: SourceRange::undefined()
                    },
                    Statement::Assignment {
                        left: Box::new(ref_to("y")),
                        right: Box::new(ref_to("x"))
                    }
                ],
                condition: Box::new(empty_stmt()),
                location: SourceRange::undefined(),
            },]
        )
    );
}

#[test]
fn test_nested_repeat_with_missing_condition_and_end_repeat() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            REPEAT
                REPEAT
                    ;
                UNTIL x = y END_REPEAT
                y := x;
            UNTIL
           END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::syntax_error("Unexpected token: 'END_PROGRAM'".into(), (171..182).into()),
            Diagnostic::missing_token("[KeywordEndRepeat]".into(), (171..182).into()),
            Diagnostic::unexpected_token_found(
                "KeywordEndRepeat".into(),
                "'END_PROGRAM'".into(),
                (171..182).into()
            ),
        ]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![Statement::RepeatLoopStatement {
                body: vec![
                    Statement::RepeatLoopStatement {
                        body: vec![empty_stmt()],
                        condition: Box::new(Statement::BinaryExpression {
                            left: Box::new(ref_to("x")),
                            right: Box::new(ref_to("y")),
                            operator: crate::ast::Operator::Equal
                        }),
                        location: SourceRange::undefined()
                    },
                    Statement::Assignment {
                        left: Box::new(ref_to("y")),
                        right: Box::new(ref_to("x"))
                    }
                ],
                condition: Box::new(empty_stmt()),
                location: SourceRange::undefined(),
            },]
        )
    );
}

#[test]
fn test_nested_repeat_with_missing_end_repeat() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            REPEAT
                REPEAT
                    ;
                UNTIL x = y END_REPEAT
                y := x;
            UNTIL x = y
           END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordEndRepeat]".into(), (177..188).into()),
            Diagnostic::unexpected_token_found(
                "KeywordEndRepeat".into(),
                "'END_PROGRAM'".into(),
                (177..188).into()
            ),
        ]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![Statement::RepeatLoopStatement {
                body: vec![
                    Statement::RepeatLoopStatement {
                        body: vec![empty_stmt()],
                        condition: Box::new(Statement::BinaryExpression {
                            left: Box::new(ref_to("x")),
                            right: Box::new(ref_to("y")),
                            operator: crate::ast::Operator::Equal
                        }),
                        location: SourceRange::undefined()
                    },
                    Statement::Assignment {
                        left: Box::new(ref_to("y")),
                        right: Box::new(ref_to("x"))
                    }
                ],
                condition: Box::new(Statement::BinaryExpression {
                    left: Box::new(ref_to("x")),
                    right: Box::new(ref_to("y")),
                    operator: crate::ast::Operator::Equal
                }),
                location: SourceRange::undefined(),
            },]
        )
    );
}

#[test]
fn test_while_with_missing_semicolon_in_body() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            WHILE x = y DO
                x := 3
            END_WHILE
            y := x;     
           END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordSemicolon, KeywordColon]".into(), (77..86).into()),
            Diagnostic::unexpected_token_found(
                "KeywordSemicolon".into(),
                "'END_WHILE'".into(),
                (77..86).into()
            ),
        ]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![
                Statement::WhileLoopStatement {
                    body: vec![Statement::Assignment {
                        left: Box::new(ref_to("x")),
                        right: Box::new(Statement::LiteralInteger {
                            value: "3".into(),
                            location: SourceRange::undefined()
                        })
                    }],
                    condition: Box::new(Statement::BinaryExpression {
                        left: Box::new(ref_to("x")),
                        right: Box::new(ref_to("y")),
                        operator: crate::ast::Operator::Equal
                    }),
                    location: SourceRange::undefined(),
                },
                Statement::Assignment {
                    left: Box::new(ref_to("y")),
                    right: Box::new(ref_to("x"))
                }
            ]
        )
    );
}

#[test]
fn test_nested_while_with_missing_end_while() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            WHILE x = y DO
                WHILE x = y DO
                    ;
                END_WHILE
                y := x;
           END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordEndWhile]".into(), (156..167).into()),
            Diagnostic::unexpected_token_found(
                "KeywordEndWhile".into(),
                "'END_PROGRAM'".into(),
                (156..167).into()
            ),
        ]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![Statement::WhileLoopStatement {
                body: vec![
                    Statement::WhileLoopStatement {
                        body: vec![empty_stmt()],
                        condition: Box::new(Statement::BinaryExpression {
                            left: Box::new(ref_to("x")),
                            right: Box::new(ref_to("y")),
                            operator: crate::ast::Operator::Equal
                        }),
                        location: SourceRange::undefined()
                    },
                    Statement::Assignment {
                        left: Box::new(ref_to("y")),
                        right: Box::new(ref_to("x"))
                    }
                ],
                condition: Box::new(Statement::BinaryExpression {
                    left: Box::new(ref_to("x")),
                    right: Box::new(ref_to("y")),
                    operator: crate::ast::Operator::Equal
                }),
                location: SourceRange::undefined(),
            },]
        )
    );
}

#[test]
fn test_while_with_missing_do() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
            WHILE x = y
                y := x;
            END_WHILE
           END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![Diagnostic::missing_token(
            "KeywordDo".into(),
            (55..56).into()
        ),]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![Statement::WhileLoopStatement {
                body: vec![Statement::Assignment {
                    left: Box::new(ref_to("y")),
                    right: Box::new(ref_to("x"))
                }],
                condition: Box::new(Statement::BinaryExpression {
                    left: Box::new(ref_to("x")),
                    right: Box::new(ref_to("y")),
                    operator: crate::ast::Operator::Equal
                }),
                location: SourceRange::undefined(),
            }]
        )
    );
}

#[test]
fn test_case_body_with_missing_semicolon() {
    //regress, this used to end in an endless loop
    let lexer = lex("PROGRAM My_PRG
           CASE x OF
           y: y := z
           END_CASE
           END_PROGRAM
    ");
    let (unit, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordSemicolon, KeywordColon]".into(), (68..76).into()),
            Diagnostic::unexpected_token_found(
                "KeywordSemicolon".into(),
                "'END_CASE'".into(),
                (68..76).into()
            ),
        ]
    );

    assert_eq!(
        format!("{:#?}", unit.implementations[0].statements),
        format!(
            "{:#?}",
            vec![Statement::CaseStatement {
                selector: Box::new(ref_to("x")),
                case_blocks: vec![ConditionalBlock {
                    condition: Box::new(ref_to("y")),
                    body: vec![Statement::Assignment {
                        left: Box::new(ref_to("y")),
                        right: Box::new(ref_to("z")),
                    }],
                },],
                else_block: vec![],
                location: SourceRange::undefined(),
            }]
        )
    );
}

#[test]
fn test_case_without_condition() {
    let lexer = lex("PROGRAM My_PRG
                CASE x OF
                    1: 
                    : x := 3;
                END_CASE
            END_PROGRAM

    ");
    let (cu, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        format!("{:#?}", cu.implementations[0].statements),
        r#"[
    CaseStatement {
        selector: Reference {
            name: "x",
        },
        case_blocks: [
            ConditionalBlock {
                condition: LiteralInteger {
                    value: "1",
                },
                body: [],
            },
            ConditionalBlock {
                condition: EmptyStatement,
                body: [
                    Assignment {
                        left: Reference {
                            name: "x",
                        },
                        right: LiteralInteger {
                            value: "3",
                        },
                    },
                ],
            },
        ],
        else_block: [],
    },
]"#
    );

    assert_eq!(
        diagnostics,
        vec![Diagnostic::syntax_error(
            "Unexpected token: ':'".into(),
            (85..86).into()
        ),]
    );
}
