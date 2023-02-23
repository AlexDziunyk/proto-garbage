use std::rc::Rc;

use crate::registry::{Field, Structure, Type};

pub fn decl_structure_to_json(structure: &Structure) -> String {
    match structure.type_name().as_str() {
        "model" => decl_model_to_json(structure),
        _ => decl_message_to_json(structure),
    }
}

pub fn def_structure_to_json(structure: &Structure) -> String {
    match structure.type_name().as_str() {
        "model" => def_model_to_json(structure),
        _ => def_message_to_json(structure),
    }
}

fn decl_message_to_json(structure: &Structure) -> String {
    format!(
        "static bool {}_to_json(u_{}_data *data, cJSON *json);",
        structure.name,
        structure.type_name()
    )
}

fn def_message_to_json(structure: &Structure) -> String {
    let mut buffer = String::default();

    buffer.push_str(&decl_message_to_json(structure).trim_end_matches(';'));
    buffer.push_str("{");
    buffer.push_str(&add_fields(
        &format!("data->{}.", &structure.name[3..]),
        &structure.fields,
    ));
    buffer.push_str("return true; }");

    buffer
}

fn decl_model_to_json(structure: &Structure) -> String {
    format!(
        "cJSON *{}_to_json({} *{});",
        structure.name, structure.cname, structure.name
    )
}

fn def_model_to_json(structure: &Structure) -> String {
    let mut buffer = String::default();

    buffer.push_str(&decl_model_to_json(structure).trim_end_matches(';'));
    buffer.push_str("{ cJSON *json = cJSON_CreateObject();");
    buffer.push_str(&add_fields(
        &format!("{}->", structure.name),
        &structure.fields,
    ));
    buffer.push_str("return json; }");

    buffer
}

fn add_fields(access: &str, fields: &[Field]) -> String {
    let mut buffer = String::default();

    fields.iter().for_each(|field| {
        buffer.push_str(&add_item(
            &field.name,
            &format!("{}{}", access, field.name),
            field.r#type.clone(),
        ))
    });

    buffer
}

fn add_item(name: &str, access: &str, r#type: Rc<Type>) -> String {
    let mut buffer = String::default();

    let access = match r#type.as_ref() {
        Type::Array(_) => {
            buffer.push_str(&create_item(name, access, r#type.clone()));
            name.to_owned()
        }
        Type::Structure(structure) => format!("{}_to_json({})", structure.name, access),
        _ => access.to_owned(),
    };

    buffer.push_str(&format!(
        "cJSON_Add{}ToObject(json, \"{}\", {});",
        r#type.json_name(),
        name,
        access
    ));

    buffer
}

fn create_item(name: &str, access: &str, r#type: Rc<Type>) -> String {
    match r#type.as_ref() {
        Type::Array(item_type) => {
            let item_str = create_item(
                "item",
                &format!("g_array_index({}, {}, i)", access, item_type.cname()),
                item_type.clone(),
            );

            format!(
                "cJSON *{name} = cJSON_CreateArray(); \
                for (guint i = 0; i < {access}->len; i++) {{ \
                    {item_str} \
                    cJSON_AddItemToArray({name}, item); \
                }}"
            )
        }
        Type::Structure(structure) => {
            format!("cJSON *{} = {}_to_json({});", name, structure.name, access)
        }
        _ => format!(
            "cJSON *{} = cJSON_Create{}({});",
            name,
            r#type.json_name(),
            access
        ),
    }
}
