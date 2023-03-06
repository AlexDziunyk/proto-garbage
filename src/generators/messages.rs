use anyhow::Result;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::{
    constants::{MESSAGES_INC_DIR, MESSAGES_SRC_DIR},
    format::format_and_write,
    registry::Structure,
};

use super::{
    json::{fields_from_json, fields_to_json},
    structure::{decl_fields, decl_params, free_fields, init_fields},
};

#[derive(Clone, Serialize)]
pub struct MessageType {
    pub name: String,
    pub prefix: String,
}

#[derive(Serialize)]
struct MessagesContext {
    message_type: MessageType,
    messages: Vec<MessageContext>,
}

#[derive(Serialize)]
struct MessageContext {
    r#type: MessageType,
    name: String,
    cname: String,
    stripped_name: String,
    fields: String,
    constructor_params: String,
    fields_init: String,
    fields_free: String,
    fields_to_json: String,
    fields_from_json: String,
}

impl MessageContext {
    fn new(r#type: &MessageType, structure: &Structure) -> Self {
        let prefix = format!("{}_", r#type.prefix);
        let stripped_name = structure.name.trim_start_matches(&prefix).to_owned();
        let access = format!("data->{}.", stripped_name);

        Self {
            r#type: r#type.clone(),
            name: structure.name.clone(),
            cname: structure.cname.clone(),
            stripped_name: stripped_name.clone(),
            fields: decl_fields(&structure.fields),
            constructor_params: decl_params(&structure.fields),
            fields_init: init_fields(
                &format!("{}->data.{}.", r#type.name, stripped_name),
                &structure.fields,
            ),
            fields_free: free_fields(&access, &structure.fields),
            fields_to_json: fields_to_json(&access, &structure.fields),
            fields_from_json: fields_from_json(&access, &structure.fields),
        }
    }
}

pub fn generate_messages(
    messages: &[Structure],
    r#type: &MessageType,
    tt: &TinyTemplate,
) -> Result<()> {
    let context = MessagesContext {
        message_type: r#type.clone(),
        messages: messages
            .iter()
            .map(|message| MessageContext::new(r#type, message))
            .collect(),
    };

    generate_messages_header(&context, tt)?;
    generate_message_source(&context, tt)?;
    for message in context.messages {
        generate_message_funcs(&message, tt)?;
    }

    Ok(())
}

fn generate_messages_header(context: &MessagesContext, tt: &TinyTemplate) -> Result<()> {
    let path = format!("{}/{}s.h", MESSAGES_INC_DIR, context.message_type.name);
    let header = tt.render("messages.h", context)?;
    format_and_write(path, &header)
}

fn generate_message_source(context: &MessagesContext, tt: &TinyTemplate) -> Result<()> {
    let path = format!("{}/{}.c", MESSAGES_SRC_DIR, context.message_type.name);
    let header = tt.render("message.c", context)?;
    format_and_write(path, &header)
}

fn generate_message_funcs(context: &MessageContext, tt: &TinyTemplate) -> Result<()> {
    let path = format!(
        "{}/{}s/{}.c",
        MESSAGES_SRC_DIR, context.r#type.name, context.stripped_name
    );
    let source = tt.render("message_funcs.c", context)?;
    format_and_write(path, &source)
}
