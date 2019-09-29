use crate::error::*;
use crate::extractors::Extension;
use crate::options::Options;
use crate::templates::Templates;
use crate::SwapData;
use actix_web::{dev::RequestHead, http, web, HttpResponse};
use serde::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::process;
use std::io::Write;

pub fn config(_options: &Options, pages: SwapData<Pages>) -> impl FnOnce(&mut web::ServiceConfig) {
    move |config| {
        config.service(
            web::resource("/{page_url:.*}")
                .guard(move |req: &RequestHead| {
                    if let Some(page) = pages.load().match_path(req.uri.path()) {
                        req.extensions.borrow_mut().insert(page);
                        true
                    } else {
                        false
                    }
                })
                .route(web::get().to(display_page)),
        );
    }
}

fn display_page(page: Extension<Arc<Page>>, pages: SwapData<Pages>) -> Result<HttpResponse> {
    let templates = &pages.load_full().templates;

    let data = PageTemplate {
        data: page.data(),
        path: &page.breadcrumb.url(),
        markdown: &page.index.markdown,
    };

    let rendered = templates.render(&page.template_name(), &data)?;

    let response = HttpResponse::Ok()
        .header(
            http::header::CONTENT_TYPE,
            http::header::ContentType::html(),
        )
        .body(rendered);

    Ok(response)
}

#[derive(Debug, Clone, Deserialize)]
struct PageIndex {
    template: PathBuf,
    #[serde(default)]
    data: PathOr<serde_yaml::Value>,
    #[serde(default)]
    markdown: HashMap<String, PathOr<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum InlinePathOr<T> {
    Path(PathBuf),
    Raw(T),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
enum PathOr<T> {
    Path(#[serde(deserialize_with = "relative_path")] PathBuf),
    Raw(T),
}

#[derive(Debug, Clone)]
struct PageIndexData {
    template: String,
    data: serde_yaml::Value,
    markdown: HashMap<String, RenderedMarkdown>,
}

#[derive(Debug, Clone, Serialize)]
struct RenderedMarkdown(String);

#[derive(Debug, Serialize)]
struct PageTemplate<'a> {
    data: &'a serde_yaml::Value,
    path: &'a str,
    markdown: &'a HashMap<String, RenderedMarkdown>,
}

#[derive(Debug, Clone)]
struct Page {
    root: PathBuf,
    index: PageIndexData,
    breadcrumb: Breadcrumb,
}

#[derive(Debug)]
pub struct Pages {
    pages: HashMap<String, Arc<Page>>,
    templates: Templates,
}

#[derive(Debug, Clone)]
struct Breadcrumb(Vec<String>);

fn relative_path<'de, D>(de: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path = PathBuf::deserialize(de)?;

    if path.to_string_lossy().contains('\n') {
        Err(serde::de::Error::custom(err!(
            "path may not contain a new-line character"
        )))
    } else if !path.is_relative() {
        Err(serde::de::Error::custom(err!("path must be relative")))
    } else {
        Ok(path)
    }
}

impl<T: Default> Default for PathOr<T> {
    fn default() -> Self {
        PathOr::Raw(T::default())
    }
}

impl Pages {
    pub fn walk_dir(mut templates: Templates, dir: &Path) -> Result<Pages> {
        let mut unvisited = vec![(dir.to_owned(), Breadcrumb(vec![]))];

        let mut pages = HashMap::new();

        while let Some((path, breadcrumb)) = unvisited.pop() {
            log::debug!("Searching {}", path.display());

            let index_path = path.join("index.yml");
            if !index_path.is_file() {
                log::debug!(
                    "Ignoring directory '{}': missing 'index.yml'",
                    index_path.display()
                );
                continue;
            }

            let index = fs::File::open(index_path)?;
            let index: PageIndex = serde_yaml::from_reader(index)?;

            let index_data = PageIndexData::compile(&path, &index)?;

            let page = Page {
                root: path,
                index: index_data,
                breadcrumb,
            };

            for entry in fs::read_dir(&page.root)? {
                let path = entry?.path();

                if path.is_dir() {
                    let dir_name = path
                        .file_name()
                        .ok_or_else(|| err!("directory has no name: {}", path.display()))?
                        .to_string_lossy()
                        .into_owned();

                    let mut crumbs = page.breadcrumb.clone();

                    crumbs.0.push(dir_name);

                    unvisited.push((path, crumbs));
                }
            }

            pages.insert(page.breadcrumb.url(), Arc::new(page));
        }

        Self::configure_templates(pages.values(), &mut templates)?;

        Ok(Pages { pages, templates })
    }

    fn configure_templates<'a>(
        pages: impl Iterator<Item = &'a Arc<Page>>,
        templates: &mut Templates,
    ) -> Result<()> {
        for page in pages {
            templates.register(&page.template_name(), page.template())?;
        }

        Ok(())
    }

    pub(self) fn match_path(&self, path: &str) -> Option<Arc<Page>> {
        self.pages.get(path).map(Arc::clone)
    }
}

impl PageIndexData {
    pub fn compile(root: &Path, index: &PageIndex) -> Result<PageIndexData> {
        let template = fs::read_to_string(root.join(&index.template))?;

        let data = match &index.data {
            PathOr::Path(path) => {
                let file = fs::File::open(root.join(path))?;
                serde_yaml::from_reader(file)?
            }
            PathOr::Raw(value) => value.clone(),
        };

        let mut markdown = HashMap::new();
        for (name, resource) in index.markdown.iter() {
            let text = match resource {
                PathOr::Path(path) => fs::read_to_string(root.join(path))?,
                PathOr::Raw(text) => text.clone(),
            };

            let rendered = Self::render_markdown(&text)?;
            markdown.insert(name.to_owned(), rendered);
        }

        let index_data = PageIndexData { template, data, markdown };

        Ok(index_data)
    }

    fn render_markdown(text: &str) -> Result<RenderedMarkdown> {
        let mut child = process::Command::new("sh")
            .arg("-c")
            .arg("pandoc --from=markdown --to=html")
            .stdin(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.as_mut().ok_or_else(|| err!("Failed to capture stdin"))?;
        stdin.write_all(text.as_bytes())?;

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let output = String::from_utf8(output.stderr)?;
            Err(err!("failed to render markdown: {}", output))
        } else {
            let output = String::from_utf8(output.stdout)?;
            Ok(RenderedMarkdown(output))
        }
    }
}

impl Page {
    pub fn template(&self) -> &str {
        &self.index.template
    }

    pub fn template_name(&self) -> String {
        format!("page_template:{}", self.breadcrumb.url())
    }

    pub fn data(&self) -> &serde_yaml::Value {
        &self.index.data
    }
}

impl Breadcrumb {
    pub fn url(&self) -> String {
        let mut url = "/".to_owned();

        for part in &self.0 {
            url.push_str(&part);
            url.push('/');
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_page_index() {
        let yaml = r#"---
template: index.hbs
data: data.yml
markdown:
    path: abc.md
    literal: |
        ---
        # Hello
        This is a paragraph
        "#;

        let index: PageIndex = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(index.template, PathBuf::from("index.hbs"));
        assert_eq!(index.data, PathOr::Path("data.yml".into()));

        let mut markdown = HashMap::new();
        markdown.insert("path".to_owned(), PathOr::Path("abc.md".into()));

        let literal = "---\n# Hello\nThis is a paragraph\n";
        markdown.insert("literal".to_owned(), PathOr::Raw(literal.to_owned()));

        assert_eq!(index.markdown, markdown);
    }

    #[test]
    fn deserialize_page_index_inline_data() {
        let yaml = r#"---
template: index.hbs
data: 
    foo: 13
    bar:
        - Jake
        - Michael
        "#;

        let index: PageIndex = serde_yaml::from_str(yaml).unwrap();

        let data = serde_yaml::from_str(r#"---
foo: 13
bar:
    - Jake
    - Michael
        "#).unwrap();

        assert_eq!(index.template, PathBuf::from("index.hbs"));
        assert_eq!(index.data, PathOr::Raw(data));
    }
}
