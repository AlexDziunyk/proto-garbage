use std::{collections::HashMap, rc::Rc};

use anyhow::{anyhow, Result};

use crate::protocol::{self, Object};

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub r#type: Rc<Type>,
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub name: String,
    pub cname: String,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub enum Type {
    Primitive(String),
    String,
    Array(Rc<Type>),
    Structure(Structure),
}

impl Type {
    pub fn cname(&self) -> String {
        match self {
            Type::Primitive(type_name) => type_name.clone(),
            Type::String => "char *".to_owned(),
            Type::Array(_) => "GArray *".to_owned(),
            Type::Structure(structure) => structure.cname.clone() + " *",
        }
    }

    pub fn json_name(&self) -> String {
        match self {
            Type::Primitive(type_name) => match type_name.as_str() {
                "bool" => "Bool",
                _ => "Number",
            },
            Type::String => "String",
            Type::Array(_) => "Item",
            Type::Structure(_) => "Item",
        }
        .to_owned()
    }
}

pub struct TypeRegistry {
    pub type_map: HashMap<String, Rc<Type>>,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        let mut registry = Self {
            type_map: HashMap::default(),
        };

        registry.register_type("int", Type::Primitive("int".to_owned()));
        registry.register_type("bool", Type::Primitive("bool".to_owned()));
        registry.register_type("float", Type::Primitive("float".to_owned()));
        registry.register_type("double", Type::Primitive("double".to_owned()));
        registry.register_type("string", Type::String);

        registry
    }
}

impl TypeRegistry {
    pub fn register_declared_type(&mut self, r#type: &protocol::Type) -> Result<()> {
        match r#type.r#type.as_str() {
            "primitive" => {
                self.register_type(&r#type.name, Type::Primitive(r#type.cname.clone()));
                Ok(())
            }
            "structure" => {
                self.register_type(
                    &r#type.name,
                    Type::Structure(Structure {
                        name: r#type.name.clone(),
                        cname: r#type.cname.clone(),
                        fields: Vec::default(),
                    }),
                );
                Ok(())
            }
            "string" | "array" => Err(anyhow!("Impossible to declare type: {}", r#type.r#type)),
            _ => Err(anyhow!("Unknown type: {}", r#type.r#type)),
        }
    }

    pub fn convert_and_register(&mut self, object: &Object) -> Structure {
        let structure = self.convert(object);

        self.register_type(&object.name, Type::Structure(structure.clone()));
        structure
    }

    pub fn convert(&mut self, object: &Object) -> Structure {
        let fields = object
            .fields
            .iter()
            .map(|object_field| Field {
                name: object_field.name.clone(),
                r#type: self.get_type(&object_field.r#type),
            })
            .collect();

        Structure {
            name: object.name.clone(),
            cname: format!("t_{}", object.name),
            fields,
        }
    }

    pub fn register_type(&mut self, name: &str, r#type: Type) {
        self.type_map.insert(name.to_owned(), Rc::new(r#type));
    }

    pub fn get_type(&self, name: &str) -> Rc<Type> {
        if &name[name.len() - 2..] == "[]" {
            let item_type = self.get_type(&name[..name.len() - 2]);
            Rc::new(Type::Array(item_type))
        } else {
            Rc::clone(&self.type_map[name])
        }
    }
}
