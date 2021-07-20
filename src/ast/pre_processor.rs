use crate::ast::UserTypeDeclaration;
// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use std::vec;

use super::super::ast::{CompilationUnit, DataType, DataTypeDeclaration, Variable};

pub fn pre_process(unit: &mut CompilationUnit) {
    //process all local variables from POUs
    for pou in unit.units.iter_mut() {
        let all_variables = pou
            .variable_blocks
            .iter_mut()
            .flat_map(|it| it.variables.iter_mut())
            .filter(|it| should_generate_implicit_type(it));

        for var in all_variables {
            pre_process_variable_data_type(pou.name.as_str(), var, &mut unit.types)
        }
    }

    //process all variables from GVLs
    let all_variables = unit
        .global_vars
        .iter_mut()
        .flat_map(|gv| gv.variables.iter_mut())
        .filter(|it| should_generate_implicit_type(it));

    for var in all_variables {
        pre_process_variable_data_type("global", var, &mut unit.types)
    }

    //process all variables in dataTypes
    let mut new_types = vec![];
    for dt in unit.types.iter_mut() {
        if let DataType::StructType { name, variables } = &mut dt.data_type {
            variables
                .iter_mut()
                .filter(|it| should_generate_implicit_type(it))
                .for_each(|var| {
                    pre_process_variable_data_type(
                        name.as_ref().unwrap().as_str(),
                        var,
                        &mut new_types,
                    )
                });
        }
    }
    unit.types.append(&mut new_types);
}

fn should_generate_implicit_type(variable: &Variable) -> bool {
    match variable.data_type {
        DataTypeDeclaration::DataTypeReference { .. } => false,
        DataTypeDeclaration::DataTypeDefinition {
            data_type: DataType::VarArgs { .. },
            ..
        } => false,
        DataTypeDeclaration::DataTypeDefinition { .. } => true,
    }
}

fn pre_process_variable_data_type(
    container_name: &str,
    variable: &mut Variable,
    types: &mut Vec<UserTypeDeclaration>,
) {
    let new_type_name = format!("__{}_{}", container_name, variable.name);
    if let DataTypeDeclaration::DataTypeDefinition { mut data_type, .. } =
        variable.replace_data_type_with_reference_to(new_type_name.clone())
    {
        // create index entry
        add_nested_datatypes(new_type_name.as_str(), &mut data_type, types);
        data_type.set_name(new_type_name);
        types.push(UserTypeDeclaration {
            data_type,
            initializer: None,
        });
    }
    //make sure it gets generated
}

fn add_nested_datatypes(
    container_name: &str,
    datatype: &mut DataType,
    types: &mut Vec<UserTypeDeclaration>,
) {
    let new_type_name = format!("{}_", container_name);
    if let Some(DataTypeDeclaration::DataTypeDefinition { mut data_type, .. }) =
        datatype.replace_data_type_with_reference_to(new_type_name.clone())
    {
        data_type.set_name(new_type_name.clone());
        add_nested_datatypes(new_type_name.as_str(), &mut data_type, types);
        types.push(UserTypeDeclaration {
            data_type,
            initializer: None,
        });
    }
}
