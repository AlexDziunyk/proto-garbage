use crate::registry::{Field, Structure};

pub fn decl_structure(structure: &Structure) -> String {
    let mut buffer = String::default();

    buffer.push_str("typedef struct {");
    structure
        .fields
        .iter()
        .for_each(|field| buffer.push_str(&decl_field(field)));
    buffer.push_str(&format!("}} {};", structure.cname));

    buffer
}

fn decl_field(field: &Field) -> String {
    format!("{} {};", field.r#type.cname(), field.name)
}
