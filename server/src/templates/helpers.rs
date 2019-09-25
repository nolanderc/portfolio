use handlebars::*;

/// Registers all helpers in this module
pub fn register_all(registry: &mut Handlebars) {
    registry.register_helper("debug", Box::new(debug));
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



