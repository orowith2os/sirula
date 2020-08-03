use libc::{LC_ALL, LC_COLLATE, LC_MESSAGES};
use std::path::Path;
use std::str::FromStr;

mod locale;
use locale::*;

use gdk::keys::constants;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{
    BoxBuilder, IconLookupFlags, IconTheme, IconThemeExt, Image, ImageBuilder, Label, LabelExt,
    Orientation, Widget,
};
use pango::EllipsizeMode;

use gio::{AppInfo, Icon};
use std::borrow::Borrow;
use std::env::args;

use gdk_pixbuf::Pixbuf;

use log::LevelFilter;
use std::io::Write;

use futures::prelude::*;

#[macro_use]
extern crate log;

struct AppEntry {
    name: String,
    info: AppInfo,
    label: Label,
    image: Image,
    hbox: gtk::Box,
    score: i32,
}

fn load_entries() -> Vec<AppEntry> {
    debug!("begin load_entries");
    let mut entries = Vec::new();

    let icon_theme = IconTheme::get_default().unwrap();
    let icon_size = 64;

    let apps = gio::AppInfo::get_all();

    debug!("got all");

    let main_context = glib::MainContext::default();

    for app in apps {
        if !app.should_show() || app.get_display_name().is_none() {
            continue;
        }
        let name = app.get_display_name().unwrap().to_string();

        let label = gtk::LabelBuilder::new().xalign(0.0f32).label(&name).build();
        label.set_line_wrap(true);
        label.set_lines(2);
        label.set_ellipsize(EllipsizeMode::End);

        let image = ImageBuilder::new().pixel_size(icon_size).build();
        if let Some(icon) = app
            .get_icon()
            .map(|icon| icon_theme.lookup_by_gicon(&icon, icon_size, IconLookupFlags::FORCE_SIZE))
            .flatten()
        {
            let image_clone = image.clone();
            main_context.spawn_local(icon.load_icon_async_future().map(move |pb| {
                if let Ok(pb) = pb {
                    image_clone.set_from_pixbuf(Some(&pb));
                }
            }));
        }

        let hbox = BoxBuilder::new()
            .orientation(Orientation::Horizontal)
            .build();
        hbox.pack_start(&image, false, false, 0);
        hbox.pack_end(&label, true, true, 0);

        entries.push(AppEntry {
            name,
            info: app,
            label,
            image,
            hbox: hbox,
            score: 100,
        });
    }

    entries.sort_by(|a, b| string_collate(&a.name, &b.name));

    debug!("built");

    entries
}

fn activate(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.connect_show(|w| {
        println!("asdf");
    });

    // window.set_size_request(1000, 500);
    // window.resize(1000, 500);

    gtk_layer_shell::init_for_window(&window);
    gtk_layer_shell::set_keyboard_interactivity(&window, true);
    gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);
    gtk_layer_shell::auto_exclusive_zone_enable(&window);

    // gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Left, 10);
    // gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Right, 10);
    // gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Top, 10);

    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Left, false);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Right, true);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Top, true);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Bottom, true);

    // Set up a widget
    // let label = gtk::Label::new(Some(""));
    // label.set_markup("<span font_desc=\"20.0\">GTK Layer Shell example!</span>");

    let vbox = gtk::BoxBuilder::new()
        .name("rootbox")
        .orientation(gtk::Orientation::Vertical)
        .build();
    let entry = gtk::EntryBuilder::new()
        .name("search")
        .width_request(200)
        .build();
    vbox.pack_start(&entry, false, false, 0);

    let scroll = gtk::ScrolledWindowBuilder::new()
        .name("scroll")
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    vbox.pack_end(&scroll, true, true, 0);

    let listbox = gtk::ListBoxBuilder::new().name("listbox").build();
    scroll.add(&listbox);

    let entries = load_entries();

    for entry in entries {
        listbox.add(&entry.hbox);
    }

    let entry2 = entry.clone();
    window.connect_key_press_event(move |w, e| {
        if !entry2.has_focus() {
            entry2.grab_focus_without_selecting();
        }
        Inhibit(match e.get_keyval() {
            constants::Escape => {
                w.close();
                true
            }
            _ => false,
        })
    });

    // let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    // let entry = gtk::SearchEntry::new();
    // entry.set_property_width_request(300);

    // hbox.pack_start(&entry, false, false, 0);

    // let scroll = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    // scroll.set_property_vscrollbar_policy(gtk::PolicyType::Never);
    // let app_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);

    // for i in 0..100 {
    //     let icon = gtk::Image::new();
    //     icon.set_from_icon_name(Some(if i % 2 == 0 {"ark"} else {"firefox"}), gtk::IconSize::Dialog);
    //     app_box.pack_end(&icon, false, false, 0);
    // }

    // scroll.add(&app_box);

    // hbox.pack_end(&scroll, true, true, 0);

    window.add(&vbox);
    window.show_all()
}

fn main() {
    let mut builder = env_logger::Builder::from_default_env();

    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{} | {} | {}",
                buf.timestamp_millis(),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)
        .init();

    env_logger::Builder::new()
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(buf, "{}: {}: {}", ts, record.level(), record.args())
        })
        .build();

    set_locale(LC_ALL, "");

    let application =
        gtk::Application::new(Some("com.subgraph.gtk-layer-example"), Default::default())
            .expect("Initialization failed...");

    application.connect_activate(|app| {
        activate(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
