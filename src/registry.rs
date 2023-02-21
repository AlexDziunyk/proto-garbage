use std::{collections::HashMap, rc::Rc};

use crate::protocol::Object;

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

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(String),
    String,
    Array(Rc<Type>),
    Structure(Structure),
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
        registry.register_type("string", Type::String);

        registry
    }
}

impl TypeRegistry {
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
                r#type: Rc::clone(&self.type_map[&object_field.r#type]),
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
        Rc::clone(&self.type_map[name])
    }
}
