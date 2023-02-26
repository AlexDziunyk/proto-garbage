use std::rc::Rc;

use crate::registry::{Field, Type};

pub fn fields_to_json(access: &str, fields: &[Field]) -> String {
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

pub fn fields_from_json(access: &str, fields: &[Field]) -> String {
    let mut get_buffer = String::default();
    let mut extract_buffer = String::default();

    fields.iter().for_each(|field| {
        get_buffer.push_str(&get_item(&field.name));
        extract_buffer.push_str(&extract_value(
            &field.name,
            &format!("{}{}", access, field.name),
            field.r#type.clone(),
        ));
    });

    get_buffer + &extract_buffer
}

fn get_item(name: &str) -> String {
    format!("cJSON *{name}_json = cJSON_GetObjectItemCaseSensitive(json, \"{name}\");")
}

fn check_type(name: &str, r#type: Rc<Type>) -> String {
    format!("cJSON_Is{}({}_json)", r#type.json_name(), name)
}

fn declare_value(name: &str, r#type: Rc<Type>) -> String {
    format!("{} {}", r#type.cname(), extract_value(name, name, r#type))
}

fn extract_value(name: &str, access: &str, r#type: Rc<Type>) -> String {
    match r#type.as_ref() {
        Type::Primitive(type_name) => {
            let value = match type_name.as_str() {
                "bool" => format!("cJSON_IsTrue({name}_json)"),
                "double" | "float" => format!("{name}_json->valuedouble"),
                _ => format!("{name}_json->valueint"),
            };
            format!("{access} = {value};")
        }
        Type::String => format!("{access} = g_strdup({name}_json->valuestring);"),
        Type::Array(item_type) => {
            let item_name = format!("{name}_item");
            let item_str = declare_value(&item_name, item_type.clone());

            format!(
                "{access} = g_array_new(false, false, sizeof({})); \
                 cJSON *{item_name}_json = NULL; \
                 cJSON_ArrayForEach({item_name}_json, {name}_json) {{ \
                    {item_str} \
                    g_array_append_val({access}, {item_name}); \
                 }}",
                item_type.cname()
            )
        }
        Type::Structure(structure) => {
            format!("{access} = {}_from_json({}_json);", structure.name, name)
        }
    }
}
