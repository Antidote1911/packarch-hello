#![allow(non_upper_case_globals)]
#![allow(clippy::arc_with_non_send_sync)]

mod actions;
mod cli;
mod cli_handler;
mod config;
mod dns;
mod embed_data;
mod gresource;
mod gui;
mod installer;
mod localization;
mod logger;
mod networkmanager;
mod pages;
mod systemd_units;
mod tweak;
mod ui;
mod utils;
mod window;

use config::APP_ID;
use utils::{PacmanWrapper, check_regular_file};
use window::HelloWindow;

use std::str;
use std::sync::{Arc, OnceLock};

use gtk::gio::prelude::*;
use gtk::prelude::*;

use clap::Parser;
use gtk::glib;

const RESPREFIX: &str = "/org/packarch/hello";

static G_HELLO_WINDOW: OnceLock<Arc<HelloWindow>> = OnceLock::new();

fn get_preferences() -> serde_json::Value {
    let pref_file = crate::embed_data::get("preferences.json").unwrap();
    let pref = std::str::from_utf8(pref_file.data.as_ref()).unwrap();
    serde_json::from_str(pref).expect("Unable to parse")
}

fn main() {
    // Setup logger.
    let _guard = logger::setup_logger();

    if std::env::args().len() > 1 {
        // Parse arguments and run CLI logic
        let cli_args = cli::Cli::parse();
        if let Err(e) = run_cli(cli_args) {
            eprintln!("Error: {e}");
        }
    } else {
        // Register UI.
        gtk::init().expect("Unable to start GTK3.");

        gresource::init().expect("Could not load gresource file.");

        // Set program name.
        glib::set_program_name("org.packarch.hello".into());
        glib::set_application_name("org.packarch.hello");

        let application = gtk::Application::new(
            Some(APP_ID),       // Application id
            Default::default(), // Using default flags
        );

        application.connect_activate(|application| {
            // If a window already exists, raise the window to the front and give it focus
            if let Some(window) = G_HELLO_WINDOW.get() {
                window.window.present();
                return;
            }
            build_ui(application);
        });

        // Run the application and start the event loop
        application.run();
    }
}

fn build_ui(application: &gtk::Application) {
    let preferences = get_preferences();

    // Init window
    let hello_window = HelloWindow::new(application, preferences);

    let builder_ref = &hello_window.builder;
    builder_ref.connect_signals(|_builder, handler_name| {
        match handler_name {
            // handler_name as defined in the glade file => handler function as defined above
            "on_action_clicked" => Box::new(on_action_clicked),
            "on_btn_clicked" => Box::new(on_btn_clicked),
            "on_link_clicked" => Box::new(on_link_clicked),
            _ => Box::new(|_| None),
        }
    });
    G_HELLO_WINDOW.set(Arc::new(hello_window)).unwrap();
}

/// Handlers
fn on_action_clicked(param: &[glib::Value]) -> Option<glib::Value> {
    let widget = param[0].get::<gtk::Widget>().unwrap();
    match widget.widget_name().as_str() {
        "install" => {
            installer::launch_installer(fl!("calamares-install-type"));
            None
        },
        "autostart" => {
            let action = widget.downcast::<gtk::Switch>().unwrap();
            G_HELLO_WINDOW.get().unwrap().set_autostart(action.is_active());
            None
        },
        _ => {
            G_HELLO_WINDOW.get().unwrap().show_about_dialog();
            None
        },
    }
}

fn on_btn_clicked(param: &[glib::Value]) -> Option<glib::Value> {
    let widget = param[0].get::<gtk::Button>().unwrap();
    let name = widget.widget_name();

    let child_name = format!("{name}page");
    G_HELLO_WINDOW.get().unwrap().set_stack_child_visible(&child_name);

    None
}

fn on_link_clicked(param: &[glib::Value]) -> Option<glib::Value> {
    let widget = param[0].get::<gtk::Widget>().unwrap();
    let name = widget.widget_name();

    let preferences = G_HELLO_WINDOW.get().unwrap().get_preferences("urls");

    let uri = preferences[name.as_str()].as_str().unwrap();
    G_HELLO_WINDOW.get().unwrap().open_uri(uri);

    None
}

fn run_cli(cli: cli::Cli) -> anyhow::Result<()> {
    match cli.command {
        cli::Commands::Fix(args) => cli_handler::handle_fix_command(args.action),
        cli::Commands::Tweak(args) => cli_handler::handle_tweak_command(args.action),
        cli::Commands::Dns(args) => cli_handler::handle_dns_command(args.action),
        cli::Commands::Launch(args) => cli_handler::handle_launch_command(args.app),
    }
}
