use crate::config::{APP_ID, VERSION};
use crate::{RESPREFIX, check_regular_file, installer, pages, utils};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;

use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib::GString;
use gtk::{Builder, HeaderBar, Window, gdk, glib};
use tracing::error;

#[derive(Clone, Debug)]
pub struct HelloWindow {
    pub builder: gtk::Builder,
    pub window: gtk::Window,
    preferences: serde_json::Value,
}

// SAFETY: GTK objects use GLib's atomic reference counting for lifetime management.
// All GTK UI calls in this application occur on the GTK main thread via GTK's own
// signal/callback system. The static is written once at startup before any reads.
unsafe impl Send for HelloWindow {}
unsafe impl Sync for HelloWindow {}

impl HelloWindow {
    /// Create a new `HelloWindow`.
    pub fn new(application: &gtk::Application, preferences: serde_json::Value) -> Self {
        // Register bundled icons with the icon theme
        if let Some(icon_theme) = gtk::IconTheme::default() {
            icon_theme.add_resource_path(&format!("{RESPREFIX}/data/img"));
        }

        // Import Css
        let provider = gtk::CssProvider::new();
        provider.load_from_resource(&format!("{RESPREFIX}/ui/style.css"));
        gtk::StyleContext::add_provider_for_screen(
            &gtk::gdk::Screen::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Init window
        let builder = Builder::from_resource(&format!("{RESPREFIX}/ui/packarch-hello.glade"));

        let main_window: Window =
            builder.object("window").expect("Could not get the object window");
        main_window.set_application(Some(application));

        // Subtitle of headerbar
        let header: HeaderBar = builder.object("headerbar").expect("Could not get the headerbar");
        header.set_subtitle(Some(&crate::fl!("headerbar-subtitle")));

        // Load images
        let logo_path = format!("{}/{APP_ID}.svg", preferences["logo_path"].as_str().unwrap());
        if Path::new(&logo_path).exists() {
            let logo = Pixbuf::from_file(logo_path).unwrap();
            main_window.set_icon(Some(&logo));
        }

        // Logo of the home page
        let distriblogo: gtk::Image = builder.object("distriblogo").unwrap();
        distriblogo.set_pixel_size(96);
        distriblogo.set_from_icon_name(Some("packarch-logo"), gtk::IconSize::Invalid);

        let homepage_grid: gtk::Grid = builder.object("homepage").unwrap();
        for widget in homepage_grid.children() {
            let casted_widget = widget.downcast::<gtk::Button>();
            if casted_widget.is_err() {
                continue;
            }

            let btn = casted_widget.unwrap();
            if btn.image_position() != gtk::PositionType::Right {
                continue;
            }
            let image = gtk::Image::from_icon_name(
                Some("external-link-symbolic"),
                gtk::IconSize::SmallToolbar,
            );
            image.set_margin_start(2);
            btn.set_image(Some(&image));
        }

        // Set autostart switcher state
        let autostart =
            Path::new(&utils::fix_path(preferences["autostart_path"].as_str().unwrap())).exists();
        let autostart_switch: gtk::Switch = builder.object("autostart").unwrap();
        autostart_switch.set_active(autostart);

        // Live systems
        if installer::is_iso(&preferences) {
            let installlabel: gtk::Label = builder.object("installlabel").unwrap();
            installlabel.set_visible(true);

            let install: gtk::Button = builder.object("install").unwrap();
            install.set_visible(true);
        } else {
            let installlabel: gtk::Label = builder.object("installlabel").unwrap();
            installlabel.set_visible(false);

            let install: gtk::Button = builder.object("install").unwrap();
            install.set_visible(false);

            pages::create_appbrowser_page(&builder);
            pages::create_tweaks_page(&builder);
            pages::create_troubleshooting_page(&builder);

            if Path::new("/usr/bin/nmcli").exists() {
                pages::dns::create_connections_page(&builder);
            }
        }

        // Show the UI
        main_window.show();

        // Set initial focus on a meaningful interactive widget
        if installer::is_iso(&preferences) {
            let install: gtk::Button = builder.object("install").unwrap();
            install.grab_focus();
        } else {
            let forum: gtk::Button = builder.object("forum").unwrap();
            forum.grab_focus();
        }

        // Briefly dim focused widget on keyboard Tab for navigation feedback
        let prev_dimmed: Rc<RefCell<Option<gtk::Widget>>> = Rc::new(RefCell::new(None));
        let prev = prev_dimmed.clone();
        let window = main_window.clone();
        main_window.connect_key_press_event(move |_, event| {
            let keyval = event.keyval();
            if keyval == gdk::keys::constants::Tab || keyval == gdk::keys::constants::ISO_Left_Tab {
                let window = window.clone();
                let prev = prev.clone();
                glib::idle_add_local_once(move || {
                    if let Some(widget) = prev.borrow_mut().take() {
                        widget.set_opacity(1.0);
                    }
                    if let Some(focused) = window.focused_widget() {
                        focused.set_opacity(0.5);
                        *prev.borrow_mut() = Some(focused.clone());
                        glib::timeout_add_local_once(Duration::from_millis(180), move || {
                            focused.set_opacity(1.0);
                        });
                    }
                });
            }
            glib::Propagation::Proceed
        });

        // setup pages content
        let hello_window = HelloWindow { window: main_window, builder, preferences };
        hello_window.set_texts();
        hello_window
    }

    pub fn show_about_dialog(&self) {
        let logo_path = format!("/usr/share/icons/hicolor/scalable/apps/{APP_ID}.svg");
        let logo = Pixbuf::from_file(logo_path).unwrap();

        let dialog = gtk::AboutDialog::builder()
        .transient_for(&self.window)
        .modal(true)
        .program_name(GString::from_string_unchecked(crate::fl!("about-dialog-title")))
        .comments(GString::from_string_unchecked(crate::fl!("about-dialog-comments")))
        .version(VERSION)
        .logo(&logo)
        .authors(vec![
            "Vladislav Nepogodin (CachyOS Hello)".to_owned(),
            "Fabrice Corraire (Packarch)".to_owned(),
        ])
        // Translators: Replace "translator-credits" with your names. Put a comma between.
        .translator_credits("translator-credits")
        .copyright("2021-2026 CachyOS team, 2026 Fabrice Corraire")
        .license_type(gtk::License::Gpl30)
        .website("https://github.com/Antidote1911/packarch-hello")
        .website_label("GitHub")
        .build();

        dialog.run();
        dialog.hide();
    }

    /// Texts of the ui (French only: the fallback language of i18n.toml)
    pub fn set_texts(&self) {
        let elts: HashMap<&str, Vec<_>> = HashMap::from([
            ("label", vec![
                "autostartlabel",
                "forum",
                "install",
                "installlabel",
                "secondcategory",
                "software",
                "welcomelabel",
                "welcometitle",
            ]),
            ("tooltip_text", vec!["about", "software", "forum"]),
        ]);

        for (method, objnames) in &elts {
            for objname in objnames {
                let item: &gtk::Widget = &self.builder.object(objname).unwrap();
                if method == &"label" {
                    let translated_text =
                        crate::localization::get_locale_text(utils::get_translation_msgid(objname));
                    item.set_property(method, &translated_text);
                } else if method == &"tooltip_text" {
                    let translated_text = if objname == &"about" {
                        crate::fl!("button-about-tooltip")
                    } else {
                        crate::fl!("button-web-resource-tooltip")
                    };
                    item.set_property(method, &translated_text);
                }
            }
        }

        pages::i18n::update_translations(&self.builder);
    }

    pub fn set_autostart(&self, autostart: bool) {
        let preferences = &self.preferences;
        let autostart_path = utils::fix_path(preferences["autostart_path"].as_str().unwrap());
        let desktop_path = preferences["desktop_path"].as_str().unwrap().to_owned();
        let config_dir = Path::new(&autostart_path).parent().unwrap();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).unwrap();
        }
        if autostart && !check_regular_file(&autostart_path) {
            std::os::unix::fs::symlink(desktop_path, &autostart_path).unwrap();
        } else if !autostart && check_regular_file(&autostart_path) {
            std::fs::remove_file(&autostart_path).unwrap();
        }
    }

    pub fn open_uri(&self, uri: &str) {
        if let Err(uri_err) = gtk::show_uri_on_window(Some(&self.window), uri, 0) {
            error!("Failed to open uri: {uri_err}");
        }
    }

    pub fn set_stack_child_visible(&self, child_name: &str) {
        let stack: &gtk::Stack = &self.builder.object("stack").unwrap();
        stack.set_visible_child_name(child_name);
    }

    pub fn get_preferences(&self, entry: &str) -> &serde_json::Value {
        &self.preferences[entry]
    }
}
