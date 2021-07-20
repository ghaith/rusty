// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use crate::compile_error::CompileError;
use std::{
    fmt::{Debug, Display, Formatter, Result},
    iter,
    ops::Range,
    result, unimplemented,
};
mod pre_processor;

#[derive(Debug, Clone, PartialEq)]
pub struct Dimension {
    pub start_offset: i32,
    pub end_offset: i32,
}

impl Dimension {
    pub fn get_length(&self) -> u32 {
        (self.end_offset - self.start_offset + 1) as u32
    }
}

#[derive(PartialEq)]
pub struct Pou {
    pub name: String,
    pub variable_blocks: Vec<VariableBlock>,
    pub pou_type: PouType,
    pub return_type: Option<DataTypeDeclaration>,
    pub location: SourceRange,
}

impl Debug for Pou {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("POU")
            .field("name", &self.name)
            .field("variable_blocks", &self.variable_blocks)
            .field("pou_type", &self.pou_type)
            .field("return_type", &self.return_type)
            .finish()
    }
}

#[derive(Debug, PartialEq)]
pub struct Implementation {
    pub name: String,
    pub type_name: String,
    pub linkage: LinkageType,
    pub pou_type: PouType,
    pub statements: Vec<Statement>,
    pub location: SourceRange,
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum LinkageType {
    Internal,
    External,
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum PouType {
    Program,
    Function,
    FunctionBlock,
    Action,
}

#[derive(Debug, PartialEq)]
pub struct CompilationUnit {
    pub global_vars: Vec<VariableBlock>,
    pub units: Vec<Pou>,
    pub implementations: Vec<Implementation>,
    pub types: Vec<UserTypeDeclaration>,
}

impl CompilationUnit {
    /// imports all elements of the other CompilationUnit into this CompilationUnit
    ///
    /// this will import all global_vars, units, implementations and types. The imported
    /// structs are moved from the other unit into this unit
    /// # Arguments
    /// `other` the other CompilationUnit to import the elements from.
    pub fn import(&mut self, other: CompilationUnit) {
        self.global_vars.extend(other.global_vars);
        self.units.extend(other.units);
        self.implementations.extend(other.implementations);
        self.types.extend(other.types);
    }
}

impl Default for CompilationUnit {
    fn default() -> Self {
        CompilationUnit {
            global_vars: Vec::new(),
            units: Vec::new(),
            implementations: Vec::new(),
            types: Vec::new(),
        }
    }
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum VariableBlockType {
    Local,
    Input,
    Output,
    Global,
    InOut,
}

#[derive(PartialEq)]
pub struct VariableBlock {
    pub variables: Vec<Variable>,
    pub variable_block_type: VariableBlockType,
}

impl Debug for VariableBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("VariableBlock")
            .field("variables", &self.variables)
            .field("variable_block_type", &self.variable_block_type)
            .finish()
    }
}

#[derive(Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub data_type: DataTypeDeclaration,
    pub initializer: Option<Statement>,
    pub location: SourceRange,
}

impl Debug for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.initializer.is_some() {
            f.debug_struct("Variable")
                .field("name", &self.name)
                .field("data_type", &self.data_type)
                .field("initializer", &self.initializer)
                .finish()
        } else {
            f.debug_struct("Variable")
                .field("name", &self.name)
                .field("data_type", &self.data_type)
                .finish()
        }
    }
}

impl Variable {
    pub fn replace_data_type_with_reference_to(
        &mut self,
        type_name: String,
    ) -> DataTypeDeclaration {
        let new_data_type = DataTypeDeclaration::DataTypeReference {
            referenced_type: type_name,
            location: self.data_type.get_location(),
        };
        std::mem::replace(&mut self.data_type, new_data_type)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceRange {
    range: core::ops::Range<usize>,
}

impl SourceRange {
    pub fn new(range: core::ops::Range<usize>) -> SourceRange {
        SourceRange { range }
    }

    pub fn undefined() -> SourceRange {
        SourceRange { range: 0..0 }
    }

    pub fn get_start(&self) -> usize {
        self.range.start
    }

    pub fn get_end(&self) -> usize {
        self.range.end
    }

    pub fn sub_range(&self, start: usize, len: usize) -> SourceRange {
        SourceRange::new((self.get_start() + start)..(self.get_start() + len))
    }

    pub fn to_range(&self) -> Range<usize> {
        self.range.clone()
    }
}

impl From<std::ops::Range<usize>> for SourceRange {
    fn from(range: std::ops::Range<usize>) -> SourceRange {
        SourceRange::new(range)
    }
}

#[derive(Clone, PartialEq)]
pub enum DataTypeDeclaration {
    DataTypeReference {
        referenced_type: String,
        location: SourceRange,
    },
    DataTypeDefinition {
        data_type: DataType,
        location: SourceRange,
    },
}

impl Debug for DataTypeDeclaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DataTypeDeclaration::DataTypeReference {
                referenced_type, ..
            } => f
                .debug_struct("DataTypeReference")
                .field("referenced_type", referenced_type)
                .finish(),
            DataTypeDeclaration::DataTypeDefinition { data_type, .. } => f
                .debug_struct("DataTypeDefinition")
                .field("data_type", data_type)
                .finish(),
        }
    }
}

impl DataTypeDeclaration {
    pub fn get_name(&self) -> Option<&str> {
        match self {
            DataTypeDeclaration::DataTypeReference {
                referenced_type, ..
            } => Some(referenced_type.as_str()),
            DataTypeDeclaration::DataTypeDefinition { data_type, .. } => data_type.get_name(),
        }
    }

    pub fn get_location(&self) -> SourceRange {
        match self {
            DataTypeDeclaration::DataTypeReference { location, .. } => location.clone(),
            DataTypeDeclaration::DataTypeDefinition { location, .. } => location.clone(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct UserTypeDeclaration {
    pub data_type: DataType,
    pub initializer: Option<Statement>,
}

#[derive(Clone, PartialEq)]
pub enum DataType {
    StructType {
        name: Option<String>, //maybe None for inline structs
        variables: Vec<Variable>,
    },
    EnumType {
        name: Option<String>, //maybe empty for inline enums
        elements: Vec<String>,
    },
    SubRangeType {
        name: Option<String>,
        referenced_type: String,
        bounds: Option<Statement>,
    },
    ArrayType {
        name: Option<String>,
        bounds: Statement,
        referenced_type: Box<DataTypeDeclaration>,
    },
    StringType {
        name: Option<String>,
        is_wide: bool, //WSTRING
        size: Option<Statement>,
    },
    VarArgs {
        referenced_type: Option<Box<DataTypeDeclaration>>,
    },
}

impl Debug for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DataType::StructType { name, variables } => f
                .debug_struct("StructType")
                .field("name", name)
                .field("variables", variables)
                .finish(),
            DataType::EnumType { name, elements } => f
                .debug_struct("EnumType")
                .field("name", name)
                .field("elements", elements)
                .finish(),
            DataType::SubRangeType {
                name,
                referenced_type,
                bounds,
            } => f
                .debug_struct("SubRangeType")
                .field("name", name)
                .field("referenced_type", referenced_type)
                .field("bounds", bounds)
                .finish(),
            DataType::ArrayType {
                name,
                bounds,
                referenced_type,
            } => f
                .debug_struct("ArrayType")
                .field("name", name)
                .field("bounds", bounds)
                .field("referenced_type", referenced_type)
                .finish(),
            DataType::StringType {
                name,
                is_wide,
                size,
            } => f
                .debug_struct("StringType")
                .field("name", name)
                .field("is_wide", is_wide)
                .field("size", size)
                .finish(),
            DataType::VarArgs { referenced_type } => f
                .debug_struct("VarArgs")
                .field("referenced_type", referenced_type)
                .finish(),
        }
    }
}

impl DataType {
    pub fn set_name(&mut self, new_name: String) {
        match self {
            DataType::StructType { name, variables: _ } => *name = Some(new_name),
            DataType::EnumType { name, elements: _ } => *name = Some(new_name),
            DataType::SubRangeType { name, .. } => *name = Some(new_name),
            DataType::ArrayType { name, .. } => *name = Some(new_name),
            DataType::StringType { name, .. } => *name = Some(new_name),
            DataType::VarArgs { .. } => {} //No names on varargs
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        match self {
            DataType::StructType { name, variables: _ } => name.as_ref().map(|x| x.as_str()),
            DataType::EnumType { name, elements: _ } => name.as_ref().map(|x| x.as_str()),
            DataType::ArrayType { name, .. } => name.as_ref().map(|x| x.as_str()),
            DataType::StringType { name, .. } => name.as_ref().map(|x| x.as_str()),
            DataType::SubRangeType { name, .. } => name.as_ref().map(|x| x.as_str()),
            DataType::VarArgs { .. } => None,
        }
    }

    //Attempts to replace the inner type with a reference. Returns the old type if replaceable
    pub fn replace_data_type_with_reference_to(
        &mut self,
        type_name: String,
    ) -> Option<DataTypeDeclaration> {
        if let DataType::ArrayType {
            referenced_type, ..
        } = self
        {
            if let DataTypeDeclaration::DataTypeReference { .. } = **referenced_type {
                return None;
            }
            let new_data_type = DataTypeDeclaration::DataTypeReference {
                referenced_type: type_name,
                location: SourceRange::undefined(), //TODO
            };
            let old_data_type = std::mem::replace(referenced_type, Box::new(new_data_type));
            Some(*old_data_type)
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ConditionalBlock {
    pub condition: Box<Statement>,
    pub body: Vec<Statement>,
}

impl Debug for ConditionalBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("ConditionalBlock")
            .field("condition", &self.condition)
            .field("body", &self.body)
            .finish()
    }
}

#[derive(Clone, PartialEq)]
pub enum Statement {
    EmptyStatement {
        location: SourceRange,
    },
    // Literals
    LiteralInteger {
        value: String,
        location: SourceRange,
    },
    LiteralDate {
        year: i32,
        month: u32,
        day: u32,
        location: SourceRange,
    },
    LiteralDateAndTime {
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        milli: u32,
        location: SourceRange,
    },
    LiteralTimeOfDay {
        hour: u32,
        min: u32,
        sec: u32,
        milli: u32,
        location: SourceRange,
    },
    LiteralTime {
        day: f64,
        hour: f64,
        min: f64,
        sec: f64,
        milli: f64,
        micro: f64,
        nano: u32,
        negative: bool,
        location: SourceRange,
    },
    LiteralReal {
        value: String,
        location: SourceRange,
    },
    LiteralBool {
        value: bool,
        location: SourceRange,
    },
    LiteralString {
        value: String,
        is_wide: bool,
        location: SourceRange,
    },
    LiteralArray {
        elements: Option<Box<Statement>>, // expression-list
        location: SourceRange,
    },
    MultipliedStatement {
        multiplier: u32,
        element: Box<Statement>,
        location: SourceRange,
    },
    // Expressions
    QualifiedReference {
        elements: Vec<Statement>,
    },
    Reference {
        name: String,
        location: SourceRange,
    },
    ArrayAccess {
        reference: Box<Statement>,
        access: Box<Statement>,
    },
    BinaryExpression {
        operator: Operator,
        left: Box<Statement>,
        right: Box<Statement>,
    },
    UnaryExpression {
        operator: Operator,
        value: Box<Statement>,
        location: SourceRange,
    },
    ExpressionList {
        expressions: Vec<Statement>,
    },
    RangeStatement {
        start: Box<Statement>,
        end: Box<Statement>,
    },
    // Assignment
    Assignment {
        left: Box<Statement>,
        right: Box<Statement>,
    },
    // OutputAssignment
    OutputAssignment {
        left: Box<Statement>,
        right: Box<Statement>,
    },
    //Call Statement
    CallStatement {
        operator: Box<Statement>,
        parameters: Box<Option<Statement>>,
        location: SourceRange,
    },
    // Control Statements
    IfStatement {
        blocks: Vec<ConditionalBlock>,
        else_block: Vec<Statement>,
        location: SourceRange,
    },
    ForLoopStatement {
        counter: Box<Statement>,
        start: Box<Statement>,
        end: Box<Statement>,
        by_step: Option<Box<Statement>>,
        body: Vec<Statement>,
        location: SourceRange,
    },
    WhileLoopStatement {
        condition: Box<Statement>,
        body: Vec<Statement>,
        location: SourceRange,
    },
    RepeatLoopStatement {
        condition: Box<Statement>,
        body: Vec<Statement>,
        location: SourceRange,
    },
    CaseStatement {
        selector: Box<Statement>,
        case_blocks: Vec<ConditionalBlock>,
        else_block: Vec<Statement>,
        location: SourceRange,
    },
    CaseCondition {
        condition: Box<Statement>,
    },
}

impl Debug for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Statement::EmptyStatement { .. } => f.debug_struct("EmptyStatement").finish(),
            Statement::LiteralInteger { value, .. } => f
                .debug_struct("LiteralInteger")
                .field("value", value)
                .finish(),
            Statement::LiteralDate {
                year, month, day, ..
            } => f
                .debug_struct("LiteralDate")
                .field("year", year)
                .field("month", month)
                .field("day", day)
                .finish(),
            Statement::LiteralDateAndTime {
                year,
                month,
                day,
                hour,
                min,
                sec,
                milli,
                ..
            } => f
                .debug_struct("LiteralDateAndTime")
                .field("year", year)
                .field("month", month)
                .field("day", day)
                .field("hour", hour)
                .field("min", min)
                .field("sec", sec)
                .field("milli", milli)
                .finish(),
            Statement::LiteralTimeOfDay {
                hour,
                min,
                sec,
                milli,
                ..
            } => f
                .debug_struct("LiteralTimeOfDay")
                .field("hour", hour)
                .field("min", min)
                .field("sec", sec)
                .field("milli", milli)
                .finish(),
            Statement::LiteralTime {
                day,
                hour,
                min,
                sec,
                milli,
                micro,
                nano,
                negative,
                ..
            } => f
                .debug_struct("LiteralTime")
                .field("day", day)
                .field("hour", hour)
                .field("min", min)
                .field("sec", sec)
                .field("milli", milli)
                .field("micro", micro)
                .field("nano", nano)
                .field("negative", negative)
                .finish(),
            Statement::LiteralReal { value, .. } => {
                f.debug_struct("LiteralReal").field("value", value).finish()
            }
            Statement::LiteralBool { value, .. } => {
                f.debug_struct("LiteralBool").field("value", value).finish()
            }
            Statement::LiteralString { value, is_wide, .. } => f
                .debug_struct("LiteralString")
                .field("value", value)
                .field("is_wide", is_wide)
                .finish(),
            Statement::LiteralArray { elements, .. } => f
                .debug_struct("LiteralArray")
                .field("elements", elements)
                .finish(),
            Statement::Reference { name, .. } => {
                f.debug_struct("Reference").field("name", name).finish()
            }
            Statement::QualifiedReference { elements, .. } => f
                .debug_struct("QualifiedReference")
                .field("elements", elements)
                .finish(),
            Statement::BinaryExpression {
                operator,
                left,
                right,
                ..
            } => f
                .debug_struct("BinaryExpression")
                .field("operator", operator)
                .field("left", left)
                .field("right", right)
                .finish(),
            Statement::UnaryExpression {
                operator, value, ..
            } => f
                .debug_struct("UnaryExpression")
                .field("operator", operator)
                .field("value", value)
                .finish(),
            Statement::ExpressionList { expressions } => f
                .debug_struct("ExpressionList")
                .field("expressions", expressions)
                .finish(),
            Statement::RangeStatement { start, end } => f
                .debug_struct("RangeStatement")
                .field("start", start)
                .field("end", end)
                .finish(),
            Statement::Assignment { left, right } => f
                .debug_struct("Assignment")
                .field("left", left)
                .field("right", right)
                .finish(),
            Statement::OutputAssignment { left, right } => f
                .debug_struct("OutputAssignment")
                .field("left", left)
                .field("right", right)
                .finish(),
            Statement::CallStatement {
                operator,
                parameters,
                ..
            } => f
                .debug_struct("CallStatement")
                .field("operator", operator)
                .field("parameters", parameters)
                .finish(),
            Statement::IfStatement {
                blocks, else_block, ..
            } => f
                .debug_struct("IfStatement")
                .field("blocks", blocks)
                .field("else_block", else_block)
                .finish(),
            Statement::ForLoopStatement {
                counter,
                start,
                end,
                by_step,
                body,
                ..
            } => f
                .debug_struct("ForLoopStatement")
                .field("counter", counter)
                .field("start", start)
                .field("end", end)
                .field("by_step", by_step)
                .field("body", body)
                .finish(),
            Statement::WhileLoopStatement {
                condition, body, ..
            } => f
                .debug_struct("WhileLoopStatement")
                .field("condition", condition)
                .field("body", body)
                .finish(),
            Statement::RepeatLoopStatement {
                condition, body, ..
            } => f
                .debug_struct("RepeatLoopStatement")
                .field("condition", condition)
                .field("body", body)
                .finish(),
            Statement::CaseStatement {
                selector,
                case_blocks,
                else_block,
                ..
            } => f
                .debug_struct("CaseStatement")
                .field("selector", selector)
                .field("case_blocks", case_blocks)
                .field("else_block", else_block)
                .finish(),
            Statement::ArrayAccess {
                reference, access, ..
            } => f
                .debug_struct("ArrayAccess")
                .field("reference", reference)
                .field("access", access)
                .finish(),
            Statement::MultipliedStatement {
                multiplier,
                element,
                ..
            } => f
                .debug_struct("MultipliedStatement")
                .field("multiplier", multiplier)
                .field("element", element)
                .finish(),
            Statement::CaseCondition { condition } => f
                .debug_struct("CaseCondition")
                .field("condition", condition)
                .finish(),
        }
    }
}

impl Statement {
    ///Returns the statement in a singleton list, or the contained statements if the statement is already a list
    pub fn get_as_list(&self) -> Vec<&Statement> {
        if let Statement::ExpressionList { expressions } = self {
            expressions.iter().collect::<Vec<&Statement>>()
        } else {
            vec![self]
        }
    }
    pub fn get_location(&self) -> SourceRange {
        match self {
            Statement::EmptyStatement { location, .. } => location.clone(),
            Statement::LiteralInteger { location, .. } => location.clone(),
            Statement::LiteralDate { location, .. } => location.clone(),
            Statement::LiteralDateAndTime { location, .. } => location.clone(),
            Statement::LiteralTimeOfDay { location, .. } => location.clone(),
            Statement::LiteralTime { location, .. } => location.clone(),
            Statement::LiteralReal { location, .. } => location.clone(),
            Statement::LiteralBool { location, .. } => location.clone(),
            Statement::LiteralString { location, .. } => location.clone(),
            Statement::LiteralArray { location, .. } => location.clone(),
            Statement::Reference { location, .. } => location.clone(),
            Statement::QualifiedReference { elements, .. } => {
                let first = elements
                    .first()
                    .map_or_else(SourceRange::undefined, |it| it.get_location());
                let last = elements
                    .last()
                    .map_or_else(SourceRange::undefined, |it| it.get_location());
                SourceRange::new(first.get_start()..last.get_end())
            }
            Statement::BinaryExpression { left, right, .. } => {
                let left_loc = left.get_location();
                let right_loc = right.get_location();
                SourceRange::new(left_loc.range.start..right_loc.range.end)
            }
            Statement::UnaryExpression { location, .. } => location.clone(),
            Statement::ExpressionList { expressions } => {
                let first = expressions
                    .first()
                    .map_or_else(SourceRange::undefined, |it| it.get_location());
                let last = expressions
                    .last()
                    .map_or_else(SourceRange::undefined, |it| it.get_location());
                SourceRange::new(first.get_start()..last.get_end())
            }
            Statement::RangeStatement { start, end } => {
                let start_loc = start.get_location();
                let end_loc = end.get_location();
                SourceRange::new(start_loc.range.start..end_loc.range.end)
            }
            Statement::Assignment { left, right } => {
                let left_loc = left.get_location();
                let right_loc = right.get_location();
                SourceRange::new(left_loc.range.start..right_loc.range.end)
            }
            Statement::OutputAssignment { left, right } => {
                let left_loc = left.get_location();
                let right_loc = right.get_location();
                SourceRange::new(left_loc.range.start..right_loc.range.end)
            }
            Statement::CallStatement { location, .. } => location.clone(),
            Statement::IfStatement { location, .. } => location.clone(),
            Statement::ForLoopStatement { location, .. } => location.clone(),
            Statement::WhileLoopStatement { location, .. } => location.clone(),
            Statement::RepeatLoopStatement { location, .. } => location.clone(),
            Statement::CaseStatement { location, .. } => location.clone(),
            Statement::ArrayAccess { reference, access } => {
                let reference_loc = reference.get_location();
                let access_loc = access.get_location();
                SourceRange::new(reference_loc.range.start..access_loc.range.end)
            }
            Statement::MultipliedStatement { location, .. } => location.clone(),
            Statement::CaseCondition { condition } => condition.get_location(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiplication,
    Division,
    Equal,
    NotEqual,
    Modulo,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    Not,
    And,
    Or,
    Xor,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let symbol = match self {
            Operator::Plus => "+",
            Operator::Minus => "-",
            Operator::Multiplication => "*",
            Operator::Division => "/",
            Operator::Equal => "=",
            _ => unimplemented!(),
        };
        f.write_str(symbol)
    }
}

/// flattens expression-lists and MultipliedStatements into a vec of statements.
/// It can also handle nested structures like 2(3(4,5))
pub fn flatten_expression_list(condition: &Statement) -> Vec<&Statement> {
    match condition {
        Statement::ExpressionList { expressions } => expressions
            .iter()
            .by_ref()
            .flat_map(|statement| flatten_expression_list(statement))
            .collect(),
        Statement::MultipliedStatement {
            multiplier,
            element,
            ..
        } => iter::repeat(flatten_expression_list(element))
            .take(*multiplier as usize)
            .flatten()
            .collect(),
        _ => vec![condition],
    }
}

pub fn pre_process(unit: &mut CompilationUnit) {
    pre_processor::pre_process(unit)
}

/// constructs a vector with all dimensions for the given bounds-statement
/// e.g. [0..10, 0..5]
pub fn get_array_dimensions(bounds: &Statement) -> result::Result<Vec<Dimension>, CompileError> {
    let mut result = vec![];
    for statement in bounds.get_as_list() {
        result.push(get_single_array_dimension(statement)?);
    }
    Ok(result)
}

/// constructs a Dimension for the given RangeStatement
/// throws an error if the given statement is no RangeStatement
fn get_single_array_dimension(bounds: &Statement) -> result::Result<Dimension, CompileError> {
    if let Statement::RangeStatement { start, end } = bounds {
        let start_offset = evaluate_constant_int(start).unwrap_or(0);
        let end_offset = evaluate_constant_int(end).unwrap_or(0);
        Ok(Dimension {
            start_offset,
            end_offset,
        })
    } else {
        Err(CompileError::codegen_error(
            format!("Unexpected Statement {:?}, expected range", bounds),
            bounds.get_location(),
        ))
    }
}

/// extracts the compile-time value of the given statement.
/// returns an error if no value can be derived at compile-time
fn extract_value(s: &Statement) -> result::Result<String, CompileError> {
    match s {
        Statement::UnaryExpression {
            operator, value, ..
        } => extract_value(value).map(|result| format!("{}{}", operator, result)),
        Statement::LiteralInteger { value, .. } => Ok(value.to_string()),
        //TODO constants
        _ => Err(CompileError::codegen_error(
            "Unsupported Statement. Cannot evaluate expression.".to_string(),
            s.get_location(),
        )),
    }
}

/// evaluate the given statemetn as i32
pub fn evaluate_constant_int(s: &Statement) -> result::Result<i32, CompileError> {
    let value = extract_value(s);
    value.map(|v| v.parse().unwrap_or(0))
}
