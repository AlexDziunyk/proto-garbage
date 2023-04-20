use std::{env, fmt::Write};

use anyhow::{anyhow, Ok, Result};
use serde_json::Value;
use tinytemplate::TinyTemplate;

use generators::{
    messages::{generate_messages, MessageType},
    models::generate_models,
};
use protocol::Protocol;
use registry::TypeRegistry;

mod constants;
mod format;
mod generators;
mod protocol;
mod registry;

fn to_uppercase(value: &Value, output: &mut String) -> tinytemplate::error::Result<()> {
    let mut temp = String::default();
    tinytemplate::format_unescaped(value, &mut temp)?;
    write!(output, "{}", temp.to_uppercase())?;
    tinytemplate::error::Result::Ok(())
}

fn load_templates() -> Result<TinyTemplate<'static>> {
    let mut tt = TinyTemplate::new();

    tt.add_template("models.h", include_str!("../templates/models.h.tmpl"))?;
    tt.add_template("model.c", include_str!("../templates/model.c.tmpl"))?;
    tt.add_template("messages.h", include_str!("../templates/messages.h.tmpl"))?;
    tt.add_template("message.c", include_str!("../templates/message.c.tmpl"))?;
    tt.add_template(
        "message_funcs.c",
        include_str!("../templates/message_funcs.c.tmpl"),
    )?;
    tt.add_template("handlers.h", include_str!("../templates/handlers.h.tmpl"))?;
    tt.add_template("handlers.c", include_str!("../templates/handlers.c.tmpl"))?;
    tt.add_template("handler.c", include_str!("../templates/handler.c.tmpl"))?;

    tt.add_formatter("to_uppercase", to_uppercase);
    tt.set_default_formatter(&tinytemplate::format_unescaped);

    Ok(tt)
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        return Err(anyhow!("1 argument expected"));
    }

    let protocol = Protocol::from_file(&args[1])?;
    let mut registry = TypeRegistry::default();

    for r#type in protocol.types.iter() {
        registry.register_declared_type(r#type)?;
    }

    let models: Vec<_> = protocol
        .models
        .iter()
        .map(|object| registry.convert_and_register(object))
        .collect();

    let tt = load_templates()?;
    generate_models(&models, &tt)?;

    let requests: Vec<_> = protocol
        .requests
        .iter()
        .map(|object| registry.convert(object))
        .collect();

    generate_messages(
        &requests,
        &MessageType {
            name: "request".to_owned(),
            prefix: "rq".to_owned(),
        },
        &tt,
    )?;

    let updates: Vec<_> = protocol
        .updates
        .iter()
        .map(|object| registry.convert(object))
        .collect();

    generate_messages(
        &updates,
        &MessageType {
            name: "update".to_owned(),
            prefix: "up".to_owned(),
        },
        &tt,
    )?;

    Ok(())
}
