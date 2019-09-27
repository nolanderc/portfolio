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
    data: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
struct PageIndexData {
    template: String,
    data: serde_yaml::Value,
}

#[derive(Debug, Serialize)]
struct PageTemplate<'a> {
    data: &'a serde_yaml::Value,
    path: &'a str,
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

            let index_data = PageIndexData::load(&path, &index)?;

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
    pub fn load(root: &Path, index: &PageIndex) -> Result<PageIndexData> {
        let template = fs::read_to_string(root.join(&index.template))?;
        let data = if let Some(path) = &index.data {
            let file = fs::File::open(root.join(path))?;
            serde_yaml::from_reader(file)?
        } else {
            serde_yaml::Value::Null
        };

        let index_data = PageIndexData { template, data };

        Ok(index_data)
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
