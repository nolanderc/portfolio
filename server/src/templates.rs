mod helpers;

use crate::error::*;
use crate::options::Options;
use handlebars::Handlebars;
use serde::*;

#[derive(Debug)]
pub struct Templates {
    hb: Handlebars,
}

impl Templates {
    pub fn new(options: &Options) -> Result<Templates> {
        let mut hb = Handlebars::new();
        helpers::register_all(&mut hb);

        hb.register_templates_directory(".hbs", &options.templates_directory)?;

        Ok(Templates { hb })
    }

    pub fn render<T>(&self, name: &str, data: &T) -> Result<String>
    where
        T: Serialize,
    {
        let text = self.hb.render(name, data)?;
        Ok(text)
    }

    pub fn register(&mut self, name: &str, text: &str) -> Result<()> {
        self.hb.register_template_string(name, text)?;
        Ok(())
    }
}
