// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use crate::ast::*;
use crate::parser::parse;
use crate::parser::Statement::LiteralInteger;
use crate::Diagnostic;
use pretty_assertions::*;

#[test]
fn empty_returns_empty_compilation_unit() {
    let (result, ..) = parse(super::lex("")).unwrap();
    assert_eq!(result.units.len(), 0);
}

#[test]
fn empty_global_vars_can_be_parsed() {
    let lexer = super::lex("VAR_GLOBAL END_VAR");
    let result = parse(lexer).unwrap().0;

    let vars = &result.global_vars[0]; //globar_vars
    let ast_string = format!("{:#?}", vars);
    let expected_ast = r#"VariableBlock {
    variables: [],
    variable_block_type: Global,
}"#;
    assert_eq!(ast_string, expected_ast)
}

#[test]
fn global_vars_can_be_parsed() {
    let lexer = super::lex("VAR_GLOBAL x : INT; y : BOOL; END_VAR");
    let result = parse(lexer).unwrap().0;

    let vars = &result.global_vars[0]; //globar_vars
    let ast_string = format!("{:#?}", vars);
    let expected_ast = r#"VariableBlock {
    variables: [
        Variable {
            name: "x",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
        Variable {
            name: "y",
            data_type: DataTypeReference {
                referenced_type: "BOOL",
            },
        },
    ],
    variable_block_type: Global,
}"#;
    assert_eq!(ast_string, expected_ast)
}

#[test]
fn two_global_vars_can_be_parsed() {
    let lexer = super::lex("VAR_GLOBAL a: INT; END_VAR VAR_GLOBAL x : INT; y : BOOL; END_VAR");
    let result = parse(lexer).unwrap().0;

    let vars = &result.global_vars; //global_vars
    let ast_string = format!("{:#?}", vars);
    let expected_ast = r#"[
    VariableBlock {
        variables: [
            Variable {
                name: "a",
                data_type: DataTypeReference {
                    referenced_type: "INT",
                },
            },
        ],
        variable_block_type: Global,
    },
    VariableBlock {
        variables: [
            Variable {
                name: "x",
                data_type: DataTypeReference {
                    referenced_type: "INT",
                },
            },
            Variable {
                name: "y",
                data_type: DataTypeReference {
                    referenced_type: "BOOL",
                },
            },
        ],
        variable_block_type: Global,
    },
]"#;
    assert_eq!(ast_string, expected_ast)
}

#[test]
fn simple_foo_program_can_be_parsed() {
    let lexer = super::lex("PROGRAM foo END_PROGRAM");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    assert_eq!(prg.pou_type, PouType::Program);
    assert_eq!(prg.name, "foo");
    assert!(prg.return_type.is_none());
}

#[test]
fn simple_foo_function_can_be_parsed() {
    let lexer = super::lex("FUNCTION foo : INT END_FUNCTION");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    assert_eq!(prg.pou_type, PouType::Function);
    assert_eq!(prg.name, "foo");

    assert_eq!(
        prg.return_type.as_ref().unwrap(),
        &DataTypeDeclaration::DataTypeReference {
            referenced_type: "INT".to_string(),
            location: (15..18).into(),
        }
    );
}

#[test]
fn simple_foo_function_block_can_be_parsed() {
    let lexer = super::lex("FUNCTION_BLOCK foo END_FUNCTION_BLOCK");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    assert_eq!(prg.pou_type, PouType::FunctionBlock);
    assert_eq!(prg.name, "foo");
    assert!(prg.return_type.is_none());
}

#[test]
fn single_action_parsed() {
    let lexer = super::lex("ACTION foo.bar END_ACTION");
    let result = parse(lexer).unwrap().0;

    let prg = &result.implementations[0];
    assert_eq!(prg.name, "foo.bar");
    assert_eq!(prg.type_name, "foo");
}

#[test]
fn two_actions_parsed() {
    let lexer = super::lex("ACTION foo.bar END_ACTION ACTION fuz.bar END_ACTION");
    let result = parse(lexer).unwrap().0;

    let prg = &result.implementations[0];
    assert_eq!(prg.name, "foo.bar");
    assert_eq!(prg.type_name, "foo");

    let prg2 = &result.implementations[1];
    assert_eq!(prg2.name, "fuz.bar");
    assert_eq!(prg2.type_name, "fuz");
}

#[test]
fn action_container_parsed() {
    let lexer = super::lex("ACTIONS foo ACTION bar END_ACTION END_ACTIONS");
    let result = parse(lexer).unwrap().0;

    let prg = &result.implementations[0];
    assert_eq!(prg.name, "foo.bar");
    assert_eq!(prg.type_name, "foo");
}

#[test]
fn two_action_containers_parsed() {
    let lexer = super::lex("ACTIONS foo ACTION bar END_ACTION ACTION buz END_ACTION END_ACTIONS");
    let result = parse(lexer).unwrap().0;

    let prg = &result.implementations[0];
    assert_eq!(prg.name, "foo.bar");
    assert_eq!(prg.type_name, "foo");

    let prg2 = &result.implementations[1];
    assert_eq!(prg2.name, "foo.buz");
    assert_eq!(prg2.type_name, "foo");
}

#[test]
fn mixed_action_types_parsed() {
    let lexer = super::lex("PROGRAM foo END_PROGRAM ACTIONS foo ACTION bar END_ACTION END_ACTIONS ACTION foo.buz END_ACTION");
    let result = parse(lexer).unwrap().0;

    let prg = &result.implementations[1];
    assert_eq!(prg.name, "foo.bar");
    assert_eq!(prg.type_name, "foo");

    let prg2 = &result.implementations[2];
    assert_eq!(prg2.name, "foo.buz");
    assert_eq!(prg2.type_name, "foo");
}

#[test]
fn actions_with_no_container_error() {
    let lexer = super::lex("ACTIONS ACTION bar END_ACTION ACTION buz END_ACTION END_ACTIONS");
    let err = parse(lexer).expect_err("Expecting parser failure");
    assert_eq!(
        err,
        Diagnostic::unexpected_token_found("Identifier".into(), "ACTION".into(), (8..14).into())
    );
}

#[test]
fn two_programs_can_be_parsed() {
    let lexer = super::lex("PROGRAM foo END_PROGRAM  PROGRAM bar END_PROGRAM");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    assert_eq!(prg.name, "foo");
    let prg2 = &result.units[1];
    assert_eq!(prg2.name, "bar");
}

#[test]
fn simple_program_with_varblock_can_be_parsed() {
    let lexer = super::lex("PROGRAM buz VAR END_VAR END_PROGRAM");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];

    assert_eq!(prg.variable_blocks.len(), 1);
}

#[test]
fn simple_program_with_two_varblocks_can_be_parsed() {
    let lexer = super::lex("PROGRAM buz VAR END_VAR VAR END_VAR END_PROGRAM");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];

    assert_eq!(prg.variable_blocks.len(), 2);
}

#[test]
fn a_program_needs_to_end_with_end_program() {
    let lexer = super::lex("PROGRAM buz ");
    let (_, diagnostics) = parse(lexer).unwrap();
    assert_eq!(
        diagnostics,
        vec![Diagnostic::unexpected_token_found(
            "KeywordEndProgram".into(),
            "''".into(),
            (12..12).into()
        ),]
    );
}

#[test]
fn a_variable_declaration_block_needs_to_end_with_endvar() {
    let lexer = super::lex("PROGRAM buz VAR END_PROGRAM ");
    let (_, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::missing_token("[KeywordEndVar]".into(), (16..27).into()),
            Diagnostic::unexpected_token_found(
                "KeywordEndVar".into(),
                "'END_PROGRAM'".into(),
                (16..27).into()
            ),
        ]
    );
}

#[test]
fn empty_statements_are_are_parsed() {
    let lexer = super::lex("PROGRAM buz ;;;; END_PROGRAM ");
    let result = parse(lexer).unwrap().0;

    let prg = &result.implementations[0];
    assert_eq!(
        format!("{:?}", prg.statements),
        format!(
            "{:?}",
            vec![
                Statement::EmptyStatement {
                    location: SourceRange::undefined()
                },
                Statement::EmptyStatement {
                    location: SourceRange::undefined()
                },
                Statement::EmptyStatement {
                    location: SourceRange::undefined()
                },
                Statement::EmptyStatement {
                    location: SourceRange::undefined()
                },
            ]
        ),
    );
}

#[test]
fn empty_statements_are_parsed_before_a_statement() {
    let lexer = super::lex("PROGRAM buz ;;;;x; END_PROGRAM ");
    let result = parse(lexer).unwrap().0;

    let prg = &result.implementations[0];

    assert_eq!(
        format!("{:?}", prg.statements),
        format!(
            "{:?}",
            vec![
                Statement::EmptyStatement {
                    location: SourceRange::undefined()
                },
                Statement::EmptyStatement {
                    location: SourceRange::undefined()
                },
                Statement::EmptyStatement {
                    location: SourceRange::undefined()
                },
                Statement::EmptyStatement {
                    location: SourceRange::undefined()
                },
                Statement::Reference {
                    name: "x".into(),
                    location: SourceRange::undefined()
                },
            ]
        ),
    );
}

#[test]
fn empty_statements_are_ignored_after_a_statement() {
    let lexer = super::lex("PROGRAM buz x;;;; END_PROGRAM ");
    let result = parse(lexer).unwrap().0;

    let prg = &result.implementations[0];
    let statement = &prg.statements[0];

    let ast_string = format!("{:#?}", statement);
    let expected_ast = r#"Reference {
    name: "x",
}"#;
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn simple_program_with_variable_can_be_parsed() {
    let lexer = super::lex("PROGRAM buz VAR x : INT; END_VAR END_PROGRAM");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    let variable_block = &prg.variable_blocks[0];
    let ast_string = format!("{:#?}", variable_block);
    let expected_ast = r#"VariableBlock {
    variables: [
        Variable {
            name: "x",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
    ],
    variable_block_type: Local,
}"#;
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn simple_program_with_var_input_can_be_parsed() {
    let lexer = super::lex("PROGRAM buz VAR_INPUT x : INT; END_VAR END_PROGRAM");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    let variable_block = &prg.variable_blocks[0];
    let ast_string = format!("{:#?}", variable_block);
    let expected_ast = r#"VariableBlock {
    variables: [
        Variable {
            name: "x",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
    ],
    variable_block_type: Input,
}"#;
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn simple_program_with_var_output_can_be_parsed() {
    let lexer = super::lex("PROGRAM buz VAR_OUTPUT x : INT; END_VAR END_PROGRAM");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    let variable_block = &prg.variable_blocks[0];
    let ast_string = format!("{:#?}", variable_block);
    let expected_ast = r#"VariableBlock {
    variables: [
        Variable {
            name: "x",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
    ],
    variable_block_type: Output,
}"#;
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn simple_program_with_var_inout_can_be_parsed() {
    let lexer = super::lex("PROGRAM buz VAR_IN_OUT x : INT; END_VAR END_PROGRAM");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    let variable_block = &prg.variable_blocks[0];
    let ast_string = format!("{:#?}", variable_block);
    let expected_ast = r#"VariableBlock {
    variables: [
        Variable {
            name: "x",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
    ],
    variable_block_type: InOut,
}"#;
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn a_function_with_varargs_can_be_parsed() {
    let lexer = super::lex("FUNCTION foo : INT VAR_INPUT x : INT; y : ...; END_VAR END_FUNCTION");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    let variable_block = &prg.variable_blocks[0];
    let ast_string = format!("{:#?}", variable_block);
    let expected_ast = r#"VariableBlock {
    variables: [
        Variable {
            name: "x",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
        Variable {
            name: "y",
            data_type: DataTypeDefinition {
                data_type: VarArgs {
                    referenced_type: None,
                },
            },
        },
    ],
    variable_block_type: Input,
}"#;
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn a_function_with_typed_varargs_can_be_parsed() {
    let lexer =
        super::lex("FUNCTION foo : INT VAR_INPUT x : INT; y : INT...; END_VAR END_FUNCTION");
    let result = parse(lexer).unwrap().0;

    let prg = &result.units[0];
    let variable_block = &prg.variable_blocks[0];
    let ast_string = format!("{:#?}", variable_block);
    let expected_ast = r#"VariableBlock {
    variables: [
        Variable {
            name: "x",
            data_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
        Variable {
            name: "y",
            data_type: DataTypeDefinition {
                data_type: VarArgs {
                    referenced_type: Some(
                        DataTypeReference {
                            referenced_type: "INT",
                        },
                    ),
                },
            },
        },
    ],
    variable_block_type: Input,
}"#;
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn simple_struct_type_can_be_parsed() {
    let (result, ..) = parse(super::lex(
        r#"
        TYPE SampleStruct :
            STRUCT
                One:INT;
                Two:INT;
                Three:INT;
            END_STRUCT
        END_TYPE 
        "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.types[0]);

    let expected_ast = format!(
        "{:#?}",
        &UserTypeDeclaration {
            data_type: DataType::StructType {
                name: Some("SampleStruct".to_string(),),
                variables: vec!(
                    Variable {
                        name: "One".to_string(),
                        data_type: DataTypeDeclaration::DataTypeReference {
                            referenced_type: "INT".to_string(),
                            location: SourceRange::undefined(),
                        },
                        initializer: None,
                        location: SourceRange::undefined(),
                    },
                    Variable {
                        name: "Two".to_string(),
                        data_type: DataTypeDeclaration::DataTypeReference {
                            referenced_type: "INT".to_string(),
                            location: SourceRange::undefined(),
                        },
                        initializer: None,
                        location: SourceRange::undefined(),
                    },
                    Variable {
                        name: "Three".to_string(),
                        data_type: DataTypeDeclaration::DataTypeReference {
                            referenced_type: "INT".to_string(),
                            location: SourceRange::undefined(),
                        },
                        initializer: None,
                        location: SourceRange::undefined(),
                    },
                ),
            },
            initializer: None,
        }
    );
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn struct_with_inline_array_can_be_parsed() {
    let (result, ..) = parse(super::lex(
        r#"
        TYPE SampleStruct :
            STRUCT
                One: ARRAY[0..1] OF INT;
            END_STRUCT
        END_TYPE 
        "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.types[0]);

    let expected_ast = r#"UserTypeDeclaration {
    data_type: StructType {
        name: Some(
            "SampleStruct",
        ),
        variables: [
            Variable {
                name: "One",
                data_type: DataTypeDefinition {
                    data_type: ArrayType {
                        name: None,
                        bounds: RangeStatement {
                            start: LiteralInteger {
                                value: "0",
                            },
                            end: LiteralInteger {
                                value: "1",
                            },
                        },
                        referenced_type: DataTypeReference {
                            referenced_type: "INT",
                        },
                    },
                },
            },
        ],
    },
    initializer: None,
}"#;
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn simple_enum_type_can_be_parsed() {
    let (result, ..) = parse(super::lex(
        r#"
        TYPE SampleEnum : (red, yellow, green);
        END_TYPE 
        "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.types[0]);

    let epxtected_ast = &UserTypeDeclaration {
        data_type: DataType::EnumType {
            name: Some("SampleEnum".to_string()),
            elements: vec!["red".to_string(), "yellow".to_string(), "green".to_string()],
        },
        initializer: None,
    };
    let expected_string = format!("{:#?}", epxtected_ast);
    assert_eq!(ast_string, expected_string);
}

#[test]
fn type_alias_can_be_parsed() {
    let (result, ..) = parse(super::lex(
        r#"
        TYPE 
            MyInt : INT;
        END_TYPE
        "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.types[0]);
    let exptected_ast = format!(
        "{:#?}",
        &UserTypeDeclaration {
            data_type: DataType::SubRangeType {
                name: Some("MyInt".to_string()),
                referenced_type: "INT".to_string(),
                bounds: None,
            },
            initializer: None,
        }
    );

    assert_eq!(ast_string, exptected_ast);
}

#[test]
fn array_type_can_be_parsed_test() {
    let (result, ..) = parse(super::lex(
        r#"
            TYPE MyArray : ARRAY[0..8] OF INT; END_TYPE
            "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.types[0]);

    let expected_ast = format!(
        "{:#?}",
        &UserTypeDeclaration {
            data_type: DataType::ArrayType {
                name: Some("MyArray".to_string()),
                bounds: Statement::RangeStatement {
                    start: Box::new(Statement::LiteralInteger {
                        value: "0".to_string(),
                        location: SourceRange::undefined(),
                    }),
                    end: Box::new(Statement::LiteralInteger {
                        value: "8".to_string(),
                        location: SourceRange::undefined(),
                    }),
                },
                referenced_type: Box::new(DataTypeDeclaration::DataTypeReference {
                    referenced_type: "INT".to_string(),
                    location: SourceRange::undefined(),
                }),
            },
            initializer: None,
        }
    );

    assert_eq!(ast_string, expected_ast);
}

#[test]
fn string_type_can_be_parsed_test() {
    let (result, ..) = parse(super::lex(
        r#"
            TYPE MyString : STRING[253]; END_TYPE
            TYPE MyString : STRING[253] := 'abc'; END_TYPE
            "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.types);

    let expected_ast = format!(
        "{:#?}",
        vec![
            UserTypeDeclaration {
                data_type: DataType::StringType {
                    name: Some("MyString".to_string()),
                    size: Some(LiteralInteger {
                        value: "253".to_string(),
                        location: (10..11).into(),
                    }),
                    is_wide: false,
                },
                initializer: None,
            },
            UserTypeDeclaration {
                data_type: DataType::StringType {
                    name: Some("MyString".to_string()),
                    size: Some(LiteralInteger {
                        value: "253".to_string(),
                        location: (10..11).into(),
                    }),
                    is_wide: false,
                },
                initializer: Some(Statement::LiteralString {
                    is_wide: false,
                    location: SourceRange::undefined(),
                    value: "abc".into(),
                }),
            }
        ]
    );

    assert_eq!(ast_string, expected_ast);
}

#[test]
fn wide_string_type_can_be_parsed_test() {
    let (result, ..) = parse(super::lex(
        r#"
            TYPE MyString : WSTRING[253]; END_TYPE
            "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.types[0]);

    let expected_ast = format!(
        "{:#?}",
        &UserTypeDeclaration {
            data_type: DataType::StringType {
                name: Some("MyString".to_string()),
                size: Some(LiteralInteger {
                    value: "253".to_string(),
                    location: (10..11).into(),
                }),
                is_wide: true,
            },
            initializer: None,
        }
    );

    assert_eq!(ast_string, expected_ast);
}

#[test]
fn array_type_initialization_with_literals_can_be_parsed_test() {
    let (result, ..) = parse(super::lex(
        r#"
            TYPE MyArray : ARRAY[0..2] OF INT := [1,2,3]; END_TYPE
            "#,
    ))
    .unwrap();

    let initializer = &result.types[0].initializer;
    let ast_string = format!("{:#?}", &initializer);

    let expected_ast = r#"Some(
    LiteralArray {
        elements: Some(
            ExpressionList {
                expressions: [
                    LiteralInteger {
                        value: "1",
                    },
                    LiteralInteger {
                        value: "2",
                    },
                    LiteralInteger {
                        value: "3",
                    },
                ],
            },
        ),
    },
)"#;
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn array_initializer_in_pou_can_be_parsed() {
    let (result, ..) = parse(super::lex(
        r#"
            PROGRAM main
            VAR
                my_array: ARRAY[0..2] OF INT := [5,6,7];
            END_VAR
            END_PROGRAM
            "#,
    ))
    .unwrap();

    let member = &result.units[0].variable_blocks[0].variables[0];
    if let Some(initializer) = &member.initializer {
        let ast_string = format!("{:#?}", initializer);
        let expected_ast = r#"LiteralArray {
    elements: Some(
        ExpressionList {
            expressions: [
                LiteralInteger {
                    value: "5",
                },
                LiteralInteger {
                    value: "6",
                },
                LiteralInteger {
                    value: "7",
                },
            ],
        },
    ),
}"#;
        assert_eq!(ast_string, expected_ast);
    } else {
        panic!("variable was not parsed as an Array");
    }
}

#[test]
fn inline_struct_declaration_can_be_parsed() {
    let (result, ..) = parse(super::lex(
        r#"
        VAR_GLOBAL
            my_struct : STRUCT
                One:INT;
                Two:INT;
                Three:INT;
            END_STRUCT
        END_VAR
        "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.global_vars[0].variables[0]);
    let expected_ast = r#"Variable {
    name: "my_struct",
    data_type: DataTypeDefinition {
        data_type: StructType {
            name: None,
            variables: [
                Variable {
                    name: "One",
                    data_type: DataTypeReference {
                        referenced_type: "INT",
                    },
                },
                Variable {
                    name: "Two",
                    data_type: DataTypeReference {
                        referenced_type: "INT",
                    },
                },
                Variable {
                    name: "Three",
                    data_type: DataTypeReference {
                        referenced_type: "INT",
                    },
                },
            ],
        },
    },
}"#;

    assert_eq!(ast_string, expected_ast);
}

#[test]
fn inline_enum_declaration_can_be_parsed() {
    let (result, ..) = parse(super::lex(
        r#"
        VAR_GLOBAL
            my_enum : (red, yellow, green);
        END_VAR
        "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.global_vars[0].variables[0]);

    let v = Variable {
        name: "my_enum".to_string(),
        data_type: DataTypeDeclaration::DataTypeDefinition {
            data_type: DataType::EnumType {
                name: None,
                elements: vec!["red".to_string(), "yellow".to_string(), "green".to_string()],
            },
            location: SourceRange::undefined(),
        },
        initializer: None,
        location: SourceRange::undefined(),
    };
    let expected_ast = format!("{:#?}", &v);
    assert_eq!(ast_string, expected_ast);
}

#[test]
fn multilevel_inline_struct_and_enum_declaration_can_be_parsed() {
    let (result, ..) = parse(super::lex(
        r#"
        VAR_GLOBAL
            my_struct : STRUCT
                    inner_enum: (red, yellow, green);
                    inner_struct: STRUCT
                        field: INT;
                    END_STRUCT
                END_STRUCT
        END_VAR
        "#,
    ))
    .unwrap();

    let ast_string = format!("{:#?}", &result.global_vars[0].variables[0]);
    let expected_ast = r#"Variable {
    name: "my_struct",
    data_type: DataTypeDefinition {
        data_type: StructType {
            name: None,
            variables: [
                Variable {
                    name: "inner_enum",
                    data_type: DataTypeDefinition {
                        data_type: EnumType {
                            name: None,
                            elements: [
                                "red",
                                "yellow",
                                "green",
                            ],
                        },
                    },
                },
                Variable {
                    name: "inner_struct",
                    data_type: DataTypeDefinition {
                        data_type: StructType {
                            name: None,
                            variables: [
                                Variable {
                                    name: "field",
                                    data_type: DataTypeReference {
                                        referenced_type: "INT",
                                    },
                                },
                            ],
                        },
                    },
                },
            ],
        },
    },
}"#;

    assert_eq!(ast_string, expected_ast);
}

#[test]
fn test_unexpected_token_error_message() {
    let source = "PROGRAM prg
                VAR ;
                END_VAR
            END_PROGRAM
    ";
    let lexer = super::lex(source);
    let (_, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        format!("{:?}", diagnostics),
        format!(
            "{:?}",
            vec![Diagnostic::unexpected_token_found(
                "KeywordEndVar".into(),
                "';'".into(),
                (32..33).into()
            ),]
        )
    );
}

#[test]
fn programs_can_be_external() {
    let lexer = super::lex("@EXTERNAL PROGRAM foo END_PROGRAM");
    let parse_result = parse(lexer).unwrap().0;
    let implementation = &parse_result.implementations[0];
    assert_eq!(LinkageType::External, implementation.linkage);
}

#[test]
fn test_unexpected_token_error_message2() {
    let lexer = super::lex(
        "SOME PROGRAM prg
                VAR ;
                END_VAR
            END_PROGRAM
    ",
    );
    let parse_result = parse(lexer);

    if let Err { 0: msg } = parse_result {
        assert_eq!(
            Diagnostic::syntax_error("Unexpected token: 'SOME'".into(), (0..4).into()),
            msg
        );
    } else {
        panic!("Expected parse error but didn't get one.");
    }
}
#[test]
fn test_unclosed_body_error_message() {
    let lexer = super::lex(
        "
            
            PROGRAM My_PRG

    ",
    );
    let (_, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        diagnostics,
        vec![Diagnostic::unexpected_token_found(
            "KeywordEndProgram".into(),
            "''".into(),
            (46..46).into()
        )]
    );
}

#[test]
fn initial_scalar_values_can_be_parsed() {
    let lexer = super::lex(
        "
            VAR_GLOBAL
                x : INT := 7;
            END_VAR

            TYPE MyStruct :
                STRUCT
                    a: INT := 69;
                    b: BOOL := TRUE;
                    c: REAL := 5.25;
                END_STRUCT
            END_TYPE

            TYPE MyInt : INT := 789;
            END_TYPE

            PROGRAM MY_PRG
                VAR
                    y : REAL := 11.3;
                END_VAR
            END_PROGRAM
            ",
    );
    let (parse_result, ..) = parse(lexer).unwrap();

    let x = &parse_result.global_vars[0].variables[0];
    let expected = r#"Variable {
    name: "x",
    data_type: DataTypeReference {
        referenced_type: "INT",
    },
    initializer: Some(
        LiteralInteger {
            value: "7",
        },
    ),
}"#;
    assert_eq!(expected, format!("{:#?}", x).as_str());

    let struct_type = &parse_result.types[0];
    let expected = r#"UserTypeDeclaration {
    data_type: StructType {
        name: Some(
            "MyStruct",
        ),
        variables: [
            Variable {
                name: "a",
                data_type: DataTypeReference {
                    referenced_type: "INT",
                },
                initializer: Some(
                    LiteralInteger {
                        value: "69",
                    },
                ),
            },
            Variable {
                name: "b",
                data_type: DataTypeReference {
                    referenced_type: "BOOL",
                },
                initializer: Some(
                    LiteralBool {
                        value: true,
                    },
                ),
            },
            Variable {
                name: "c",
                data_type: DataTypeReference {
                    referenced_type: "REAL",
                },
                initializer: Some(
                    LiteralReal {
                        value: "5.25",
                    },
                ),
            },
        ],
    },
    initializer: None,
}"#;
    assert_eq!(expected, format!("{:#?}", struct_type).as_str());

    let my_int_type = &parse_result.types[1];
    let expected = r#"UserTypeDeclaration {
    data_type: SubRangeType {
        name: Some(
            "MyInt",
        ),
        referenced_type: "INT",
        bounds: None,
    },
    initializer: Some(
        LiteralInteger {
            value: "789",
        },
    ),
}"#;
    assert_eq!(expected, format!("{:#?}", my_int_type).as_str());

    let y = &parse_result.units[0].variable_blocks[0].variables[0];
    let expected = r#"Variable {
    name: "y",
    data_type: DataTypeReference {
        referenced_type: "REAL",
    },
    initializer: Some(
        LiteralReal {
            value: "11.3",
        },
    ),
}"#;

    assert_eq!(expected, format!("{:#?}", y).as_str());
}

#[test]
fn array_initializer_can_be_parsed() {
    let lexer = super::lex(
        "
            VAR_GLOBAL
                x : ARRAY[0..2] OF INT := [7,8,9];
            END_VAR
           ",
    );
    let (parse_result, ..) = parse(lexer).unwrap();
    let x = &parse_result.global_vars[0].variables[0];
    let expected = r#"Variable {
    name: "x",
    data_type: DataTypeDefinition {
        data_type: ArrayType {
            name: None,
            bounds: RangeStatement {
                start: LiteralInteger {
                    value: "0",
                },
                end: LiteralInteger {
                    value: "2",
                },
            },
            referenced_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
    },
    initializer: Some(
        LiteralArray {
            elements: Some(
                ExpressionList {
                    expressions: [
                        LiteralInteger {
                            value: "7",
                        },
                        LiteralInteger {
                            value: "8",
                        },
                        LiteralInteger {
                            value: "9",
                        },
                    ],
                },
            ),
        },
    ),
}"#;
    assert_eq!(expected, format!("{:#?}", x).as_str());
}

#[test]
fn multi_dim_array_initializer_can_be_parsed() {
    let lexer = super::lex(
        "
            VAR_GLOBAL
                x : MyMultiArray := [[1,2],[3,4],[5,6]];
            END_VAR
           ",
    );
    let (parse_result, ..) = parse(lexer).unwrap();
    let x = &parse_result.global_vars[0].variables[0];
    let expected = r#"Variable {
    name: "x",
    data_type: DataTypeReference {
        referenced_type: "MyMultiArray",
    },
    initializer: Some(
        LiteralArray {
            elements: Some(
                ExpressionList {
                    expressions: [
                        LiteralArray {
                            elements: Some(
                                ExpressionList {
                                    expressions: [
                                        LiteralInteger {
                                            value: "1",
                                        },
                                        LiteralInteger {
                                            value: "2",
                                        },
                                    ],
                                },
                            ),
                        },
                        LiteralArray {
                            elements: Some(
                                ExpressionList {
                                    expressions: [
                                        LiteralInteger {
                                            value: "3",
                                        },
                                        LiteralInteger {
                                            value: "4",
                                        },
                                    ],
                                },
                            ),
                        },
                        LiteralArray {
                            elements: Some(
                                ExpressionList {
                                    expressions: [
                                        LiteralInteger {
                                            value: "5",
                                        },
                                        LiteralInteger {
                                            value: "6",
                                        },
                                    ],
                                },
                            ),
                        },
                    ],
                },
            ),
        },
    ),
}"#;
    assert_eq!(expected, format!("{:#?}", x).as_str());
}

#[test]
fn array_initializer_multiplier_can_be_parsed() {
    let lexer = super::lex(
        "
            VAR_GLOBAL
                x : ARRAY[0..2] OF INT := [3(7)];
            END_VAR
           ",
    );
    let (parse_result, ..) = parse(lexer).unwrap();
    let x = &parse_result.global_vars[0].variables[0];
    let expected = r#"Variable {
    name: "x",
    data_type: DataTypeDefinition {
        data_type: ArrayType {
            name: None,
            bounds: RangeStatement {
                start: LiteralInteger {
                    value: "0",
                },
                end: LiteralInteger {
                    value: "2",
                },
            },
            referenced_type: DataTypeReference {
                referenced_type: "INT",
            },
        },
    },
    initializer: Some(
        LiteralArray {
            elements: Some(
                MultipliedStatement {
                    multiplier: 3,
                    element: LiteralInteger {
                        value: "7",
                    },
                },
            ),
        },
    ),
}"#;
    assert_eq!(expected, format!("{:#?}", x).as_str());
}

#[test]
fn struct_initializer_can_be_parsed() {
    let lexer = super::lex(
        "
            VAR_GLOBAL
                x : Point := (x := 1, y:= 2);
            END_VAR
           ",
    );
    let (parse_result, ..) = parse(lexer).unwrap();
    let x = &parse_result.global_vars[0].variables[0];
    let expected = r#"Variable {
    name: "x",
    data_type: DataTypeReference {
        referenced_type: "Point",
    },
    initializer: Some(
        ExpressionList {
            expressions: [
                Assignment {
                    left: Reference {
                        name: "x",
                    },
                    right: LiteralInteger {
                        value: "1",
                    },
                },
                Assignment {
                    left: Reference {
                        name: "y",
                    },
                    right: LiteralInteger {
                        value: "2",
                    },
                },
            ],
        },
    ),
}"#;
    assert_eq!(expected, format!("{:#?}", x).as_str());
}

#[test]
fn string_variable_declaration_can_be_parsed() {
    let lexer = super::lex(
        "
            VAR_GLOBAL
                x : STRING;
                y : STRING[500];
                wx : WSTRING;
                wy : WSTRING[500];
            END_VAR
           ",
    );
    let (parse_result, ..) = parse(lexer).unwrap();
    let x = &parse_result.global_vars[0].variables[0];
    let expected = r#"Variable {
    name: "x",
    data_type: DataTypeDefinition {
        data_type: StringType {
            name: None,
            is_wide: false,
            size: None,
        },
    },
}"#;
    assert_eq!(expected, format!("{:#?}", x).as_str());

    let x = &parse_result.global_vars[0].variables[1];
    let expected = r#"Variable {
    name: "y",
    data_type: DataTypeDefinition {
        data_type: StringType {
            name: None,
            is_wide: false,
            size: Some(
                LiteralInteger {
                    value: "500",
                },
            ),
        },
    },
}"#;
    assert_eq!(expected, format!("{:#?}", x).as_str());

    let x = &parse_result.global_vars[0].variables[2];
    let expected = r#"Variable {
    name: "wx",
    data_type: DataTypeDefinition {
        data_type: StringType {
            name: None,
            is_wide: true,
            size: None,
        },
    },
}"#;
    assert_eq!(expected, format!("{:#?}", x).as_str());

    let x = &parse_result.global_vars[0].variables[3];
    let expected = r#"Variable {
    name: "wy",
    data_type: DataTypeDefinition {
        data_type: StringType {
            name: None,
            is_wide: true,
            size: Some(
                LiteralInteger {
                    value: "500",
                },
            ),
        },
    },
}"#;
    assert_eq!(expected, format!("{:#?}", x).as_str());
}

#[test]
fn subrangetype_can_be_parsed() {
    let lexer = super::lex(
        "
            VAR_GLOBAL
                x : UINT(0..1000);
            END_VAR
           ",
    );
    let (parse_result, ..) = parse(lexer).unwrap();

    let x = &parse_result.global_vars[0].variables[0];
    let expected = Variable {
        name: "x".to_string(),
        data_type: DataTypeDeclaration::DataTypeDefinition {
            data_type: DataType::SubRangeType {
                name: None,
                bounds: Some(Statement::RangeStatement {
                    start: Box::new(LiteralInteger {
                        value: "0".to_string(),
                        location: SourceRange::undefined(),
                    }),
                    end: Box::new(LiteralInteger {
                        value: "1000".to_string(),
                        location: SourceRange::undefined(),
                    }),
                }),
                referenced_type: "UINT".to_string(),
            },
            location: SourceRange::undefined(),
        },
        initializer: None,
        location: (0..0).into(),
    };
    assert_eq!(format!("{:#?}", expected), format!("{:#?}", x).as_str());
}

#[test]
fn varargs_parameters_can_be_parsed() {
    let lexer = super::lex(
        "
            FUNCTION foo : DINT
            VAR_INPUT
            args1 : ...;
            args2 : INT...;
            END_VAR
            END_FUNCTION
           ",
    );
    let (parse_result, diagnostics) = parse(lexer).unwrap();

    assert_eq!(
        format!("{:#?}", diagnostics),
        format!("{:#?}", Vec::<Diagnostic>::new()).as_str()
    );

    let x = &parse_result.units[0];
    let expected = Pou {
        name: "foo".into(),
        pou_type: PouType::Function,
        return_type: Some(DataTypeDeclaration::DataTypeReference {
            referenced_type: "DINT".into(),
            location: SourceRange::undefined(),
        }),
        variable_blocks: vec![VariableBlock {
            variable_block_type: VariableBlockType::Input,
            variables: vec![
                Variable {
                    name: "args1".into(),
                    data_type: DataTypeDeclaration::DataTypeDefinition {
                        data_type: DataType::VarArgs {
                            referenced_type: None,
                        },
                        location: SourceRange::undefined(),
                    },
                    initializer: None,
                    location: SourceRange::undefined(),
                },
                Variable {
                    name: "args2".into(),
                    data_type: DataTypeDeclaration::DataTypeDefinition {
                        data_type: DataType::VarArgs {
                            referenced_type: Some(Box::new(
                                DataTypeDeclaration::DataTypeReference {
                                    referenced_type: "INT".into(),
                                    location: SourceRange::undefined(),
                                },
                            )),
                        },
                        location: SourceRange::undefined(),
                    },
                    initializer: None,
                    location: SourceRange::undefined(),
                },
            ],
        }],
        location: SourceRange::undefined(),
    };
    assert_eq!(format!("{:#?}", expected), format!("{:#?}", x).as_str());
}
