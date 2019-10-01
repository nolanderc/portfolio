mod error;
mod extractors;
mod options;
mod pages;
mod templates;

use crate::error::*;
use crate::options::Options;
use crate::pages::Pages;
use crate::templates::Templates;
use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpServer};
use arc_swap::ArcSwap;
use notify::{raw_watcher, RecursiveMode, Watcher};
use std::path::Path;
use std::process;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

pub type SwapData<T> = web::Data<ArcSwap<T>>;

fn main() {
    env_logger::init();

    let options = Box::new(Options::from_args());
    let options = Box::leak(options);

    match start_server(options) {
        Err(e) => {
            log::error!("Fatal: {}", e);
            process::exit(1);
        }
        Ok(()) => (),
    }
}

fn start_server(options: &'static Options) -> Result<()> {
    let pages = init_pages(options)?;

    let pages = Arc::new(pages);
    let pages = web::Data::new(ArcSwap::new(pages));

    if options.watch {
        let dirs = [&options.pages_directory, &options.templates_directory];
        let handler = reload_pages(options, pages.clone());
        let delay = Duration::from_secs_f32(options.delay);
        watch_directories(&dirs, delay, handler)?;
    }

    let app = move || {
        App::new()
            .register_data(pages.clone())
            .configure(pages::config(options, pages.clone()))
            .service(Files::new("/", &options.pages_directory))
            .default_service(web::route().to(not_found))
    };

    let server = HttpServer::new(app).bind((options.address, options.port))?;

    server.run()?;

    Ok(())
}

fn not_found(req: HttpRequest) -> String {
    format!("404 Not Found: '{}'", req.path())
}

fn watch_directories(
    directories: impl IntoIterator<Item = impl AsRef<Path>>,
    delay: Duration,
    mut f: impl FnMut() -> bool + Send + Sync + 'static,
) -> Result<()> {
    let (tx, events) = mpsc::channel();

    let mut watcher = raw_watcher(tx)?;

    for dir in directories {
        watcher.watch(dir, RecursiveMode::Recursive)?;
    }

    let handler = move || {
        let _watcher = watcher;

        while let Ok(_) = events.recv() {
            log::info!("file changes detected");

            thread::sleep(delay);

            // ignore events that occured during the delay
            while events.try_recv().is_ok() {}

            if !f() {
                break;
            }
        }

        log::info!("file watcher shutting down...");
    };

    thread::spawn(handler);

    Ok(())
}

fn reload_pages(options: &'static Options, pages: SwapData<Pages>) -> impl FnMut() -> bool {
    move || {
        log::info!("reloading pages");

        let new_pages = match init_pages(options) {
            Ok(data) => data,
            Err(e) => {
                log::error!("file watcher encountered an error: {}", e);
                return true;
            }
        };

        let new_pages = Arc::from(new_pages);

        log::info!("swapping in new pages");
        pages.store(new_pages);

        true
    }
}

fn init_pages(options: &'static Options) -> Result<Pages> {
    let templates = Templates::new(options)?;
    let pages = Pages::walk_dir(templates, &options.pages_directory)?;

    Ok(pages)
}
