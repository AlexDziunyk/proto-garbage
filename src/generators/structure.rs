use std::rc::Rc;

use crate::registry::{Field, Type};

pub fn free_fields(access: &str, fields: &[Field]) -> String {
    let mut buffer = String::default();

    fields.iter().for_each(|field| {
        buffer.push_str(&free_value(
            &format!("{}{}", access, field.name),
            field.r#type.clone(),
        ))
    });

    buffer
}

pub fn decl_fields(fields: &[Field]) -> String {
    let mut buffer = String::default();

    fields
        .iter()
        .for_each(|field| buffer.push_str(&decl_field(field)));

    buffer
}

fn decl_field(field: &Field) -> String {
    format!("{} {};", field.r#type.cname(), field.name)
}

fn free_value(access: &str, r#type: Rc<Type>) -> String {
    match r#type.as_ref() {
        Type::Primitive(_) => String::default(),
        Type::String => format!("g_free({access});"),
        Type::Array(item_type) => {
            let item_str = free_value(
                &format!("g_array_index({}, {}, i)", access, item_type.cname()),
                item_type.clone(),
            );

            let loop_str = if !item_str.is_empty() {
                format!(
                    "for (guint i = 0; i < {access}->len; i++) {{ \
                        {item_str} \
                    }}"
                )
            } else {
                String::default()
            };

            format!(
                "{loop_str} \
                g_array_free({access}, true);"
            )
        }
        Type::Structure(structure) => format!("free_{}({});", structure.name, access),
    }
}
