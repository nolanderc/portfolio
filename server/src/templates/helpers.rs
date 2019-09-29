use handlebars::*;
use serde_json::Value as Json;

/// Registers all helpers in this module
pub fn register_all(registry: &mut Handlebars) {
    registry.register_helper("debug", Box::new(debug));
    registry.register_helper("access", Box::new(access));
}

fn debug(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    h.params().iter().try_for_each(|param| -> HelperResult {
        out.write(r#"<pre style="text-align:left;"><code>"#)?;
        if let Some(path) = param.path() {
            out.write(&format!("{} = ", path))?;
        }

        let text = serde_json::to_string_pretty(param.value())?;
        out.write(&text)?;
        out.write("</code></pre>")?;

        Ok(())
    })
}

fn access(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let source = h
        .param(0)
        .ok_or_else(|| RenderError::new("Missing data parameter"))?;
    let path = h
        .param(1)
        .ok_or_else(|| RenderError::new("Missing path parameter"))?
        .value()
        .as_str()
        .ok_or_else(|| RenderError::new("Expected a string"))?;

    let components = path.split(':');

    let mut current = source.value();
    for component in components {
        current = &current[component];
    }

    match dbg!(current) {
        Json::Null => (),
        Json::String(text) => out.write(text)?,
        Json::Number(number) => out.write(&number.to_string())?,
        Json::Bool(true) => out.write("true")?,
        Json::Bool(false) => out.write("false")?,
        current => {
            out.write(r#"<pre style="text-align:left;"><code>"#)?;
            let text = serde_json::to_string_pretty(current)?;
            out.write(&text)?;
            out.write("</code></pre>")?;
        },
    }

    Ok(())
}
