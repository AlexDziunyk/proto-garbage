use anyhow::Result;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::{
    constants::{MODELS_INC_DIR, MODELS_SRC_DIR},
    format::format_and_write,
    registry::Structure,
};

use super::{
    json::{fields_from_json, fields_to_json},
    structure::{decl_fields, free_fields},
};

#[derive(Serialize)]
struct ModelsContext {
    models: Vec<ModelContext>,
}

#[derive(Serialize)]
struct ModelContext {
    name: String,
    cname: String,
    fields: String,
    fields_free: String,
    fields_to_json: String,
    fields_from_json: String,
}

impl ModelContext {
    fn from_structure(structure: &Structure) -> Self {
        let access = format!("{}->", structure.name);

        Self {
            name: structure.name.clone(),
            cname: structure.cname.clone(),
            fields: decl_fields(&structure.fields),
            fields_free: free_fields(&access, &structure.fields),
            fields_to_json: fields_to_json(&access, &structure.fields),
            fields_from_json: fields_from_json(&access, &structure.fields),
        }
    }
}

pub fn generate_models(models: &[Structure], tt: &TinyTemplate) -> Result<()> {
    let context = ModelsContext {
        models: models.iter().map(ModelContext::from_structure).collect(),
    };

    generate_models_header(&context, tt)?;
    for model in context.models {
        generate_model_source(&model, tt)?;
    }

    Ok(())
}

fn generate_models_header(context: &ModelsContext, tt: &TinyTemplate) -> Result<()> {
    let path = format!("{}/models.h", MODELS_INC_DIR);
    let header = tt.render("models.h", context)?;
    format_and_write(path, &header)
}

fn generate_model_source(context: &ModelContext, tt: &TinyTemplate) -> Result<()> {
    let path = format!("{}/{}.c", MODELS_SRC_DIR, context.name);
    let source = tt.render("model.c", context)?;
    format_and_write(path, &source)
}
