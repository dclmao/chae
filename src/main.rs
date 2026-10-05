use adw::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Category {
    Shell,
    Icons,
    Cursors,
}

impl Category {
    const ALL: [Self; 3] = [Self::Shell, Self::Icons, Self::Cursors];

    fn label(self) -> &'static str {
        match self {
            Self::Shell => "Shell themes",
            Self::Icons => "Application icons",
            Self::Cursors => "Mouse cursors",
        }
    }

    fn settings_key(self) -> &'static str {
        match self {
            Self::Shell => "name",
            Self::Icons => "icon-theme",
            Self::Cursors => "cursor-theme",
        }
    }

    fn settings_schema(self) -> &'static str {
        match self {
            Self::Shell => "org.gnome.shell.extensions.user-theme",
            Self::Icons | Self::Cursors => "org.gnome.desktop.interface",
        }
    }

    fn default_theme(self) -> &'static str {
        match self {
            Self::Shell | Self::Icons => "Adwaita",
            Self::Cursors => "default",
        }
    }

    fn accepts(self, path: &Path) -> bool {
        match self {
            Self::Shell => path.join("gnome-shell").is_dir() || path.join("index.theme").is_file(),
            Self::Icons => path.join("index.theme").is_file() && !path.join(".cursor").is_file(),
            Self::Cursors => path.join("index.theme").is_file() && path.join(".cursor").is_file(),
        }
    }
}

fn main() {
    adw::init().expect("Failed to initialize libadwaita");

    let app = adw::Application::builder()
        .application_id("io.github.chae.AppearanceManager")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Appearance")
        .default_width(760)
        .default_height(620)
        .build();

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&gtk::Label::new(Some("Appearance"))));

    let menu_button = gtk::MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    menu_button.set_tooltip_text(Some("Application menu"));

    let menu = gio::Menu::new();
    menu.append(Some("Browse"), Some("app.browse"));
    menu.append(Some("About"), Some("app.about"));
    menu_button.set_menu_model(Some(&menu));
    header.pack_start(&menu_button);

    let browse_action = gio::SimpleAction::new("browse", None);
    browse_action.connect_activate(|_, _| {
        let _ = gtk::gio::AppInfo::launch_default_for_uri(
            "http://gnome-look.org",
            None::<&gtk::gio::AppLaunchContext>,
        );
    });
    app.add_action(&browse_action);

    let about_action = gio::SimpleAction::new("about", None);
    let app_for_about = app.clone();
    about_action.connect_activate(move |_, _| {
        let about = adw::AboutWindow::builder()
            .application_name("Chae")
            .application_icon("preferences-desktop-theme")
            .version(env!("CARGO_PKG_VERSION"))
            .developer_name("Chae contributors")
            .license_type(gtk::License::Gpl30)
            .comments("A local GNOME appearance manager for themes, icons, and cursors.")
            .website("http://gnome-look.org")
            .build();
        if let Some(parent) = app_for_about.active_window() {
            about.set_transient_for(Some(&parent));
        }
        about.present();
    });
    app.add_action(&about_action);

    let category_switcher = gtk::StackSwitcher::new();
    let category_stack = gtk::Stack::new();
    category_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    category_stack.set_transition_duration(160);
    category_switcher.set_stack(Some(&category_stack));

    let category_bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    category_bar.add_css_class("toolbar");
    category_bar.set_halign(gtk::Align::Center);
    category_bar.append(&category_switcher);

    let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_button.set_tooltip_text(Some("Refresh themes"));
    refresh_button.set_widget_name("refresh-themes-button");
    refresh_button.add_css_class("flat");
    category_bar.append(&refresh_button);

    let content_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content_box.set_margin_top(24);
    content_box.set_margin_bottom(24);
    content_box.set_margin_start(28);
    content_box.set_margin_end(28);
    content_box.set_valign(gtk::Align::Start);

    let intro = gtk::Label::builder()
        .label("Themes installed on this system")
        .xalign(0.0)
        .wrap(true)
        .build();
    intro.add_css_class("title-2");

    let description = gtk::Label::builder()
        .label("Browse local theme, icon, and cursor directories. Nothing is downloaded.")
        .xalign(0.0)
        .wrap(true)
        .build();
    description.add_css_class("dim-label");

    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);

    let status = gtk::Label::builder().xalign(0.0).wrap(true).build();
    status.add_css_class("dim-label");

    content_box.append(&intro);
    content_box.append(&description);
    content_box.append(&list);
    content_box.append(&status);

    let scrolled_content = gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&content_box)
        .build();

    let content_page = adw::ToolbarView::new();
    content_page.add_top_bar(&category_bar);
    content_page.set_content(Some(&scrolled_content));

    for category in Category::ALL {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
        page.set_hexpand(true);
        page.set_vexpand(true);
        category_stack.add_titled(&page, Some(category.settings_key()), category.label());
    }

    let category_stack_page = adw::NavigationPage::new(&content_page, "Appearance");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.append(&header);
    root.set_vexpand(true);
    root.append(&category_stack_page);
    window.set_content(Some(&root));

    let list_for_selection = list.clone();
    let status_for_selection = status.clone();
    category_stack.connect_visible_child_notify(move |stack| {
        let category = match stack.visible_child_name().as_deref() {
            Some("icon-theme") => Category::Icons,
            Some("cursor-theme") => Category::Cursors,
            _ => Category::Shell,
        };
        populate_list(category, &list_for_selection, &status_for_selection);
    });
    category_stack.set_visible_child_name(Category::Shell.settings_key());
    populate_list(Category::Shell, &list, &status);

    let list_for_refresh = list.clone();
    let status_for_refresh = status.clone();
    let stack_for_refresh = category_stack.clone();
    refresh_button.connect_clicked(move |_| {
        let category = match stack_for_refresh.visible_child_name().as_deref() {
            Some("icon-theme") => Category::Icons,
            Some("cursor-theme") => Category::Cursors,
            _ => Category::Shell,
        };
        populate_list(category, &list_for_refresh, &status_for_refresh);
    });

    window.present();
}

fn populate_list(category: Category, list: &gtk::ListBox, status: &gtk::Label) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    let mut themes = discover(category);
    themes.retain(|path| {
        path.file_name()
            .is_some_and(|name| !name.to_string_lossy().eq_ignore_ascii_case("hicolor"))
    });
    themes.sort_by_key(|path| path.file_name().map(|n| n.to_string_lossy().to_lowercase()));

    let current = current_theme(category);
    let default_name = category.default_theme();
    let default_row = adw::ActionRow::builder()
        .title("Default")
        .subtitle(if current.as_deref() == Some(default_name) {
            "Currently selected · Adwaita"
        } else {
            "Use the GNOME default appearance"
        })
        .build();
    let default_apply = gtk::Button::with_label("Apply");
    default_apply.add_css_class("suggested-action");
    default_apply.set_valign(gtk::Align::Center);
    let status_for_default = status.clone();
    default_apply.connect_clicked(move |_| match apply_theme(category, default_name) {
        Ok(()) => status_for_default.set_text("Applied the default appearance."),
        Err(error) => status_for_default.set_text(&error),
    });
    default_row.add_suffix(&default_apply);
    list.append(&default_row);

    for path in &themes {
        let Some(name) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            continue;
        };
        let row = adw::ActionRow::builder().title(&name).build();
        if current.as_deref() == Some(name.as_str()) {
            row.set_subtitle("Currently selected");
        }
        let apply = gtk::Button::with_label("Apply");
        apply.add_css_class("suggested-action");
        apply.set_valign(gtk::Align::Center);
        let name_for_apply = name.clone();
        let status_for_apply = status.clone();
        apply.connect_clicked(move |_| match apply_theme(category, &name_for_apply) {
            Ok(()) => status_for_apply.set_text(&format!("Applied ‘{name_for_apply}’.")),
            Err(error) => status_for_apply.set_text(&error),
        });
        row.add_suffix(&apply);
        list.append(&row);
    }

    if themes.is_empty() {
        let row = adw::ActionRow::builder()
            .title("No additional themes found")
            .subtitle("Only the default appearance is available.")
            .build();
        list.append(&row);
    }

    status.set_text(&format!(
        "{} additional themes found. Directories checked: {}",
        themes.len(),
        search_directories(category)
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));
}

fn discover(category: Category) -> Vec<PathBuf> {
    let mut results = Vec::new();
    for base in search_directories(category) {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && category.accepts(&path) {
                results.push(path);
            }
        }
    }
    results
}

fn search_directories(category: Category) -> Vec<PathBuf> {
    let mut bases = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        match category {
            Category::Shell => {
                bases.push(home.join(".local/share/themes"));
                bases.push(home.join(".themes"));
            }
            Category::Icons | Category::Cursors => {
                bases.push(home.join(".icons"));
                bases.push(home.join(".local/share/icons"));
            }
        }
    }

    match category {
        Category::Shell => {
            bases.push(PathBuf::from("/usr/local/share/themes"));
            bases.push(PathBuf::from("/usr/share/themes"));
        }
        Category::Icons => {
            bases.push(PathBuf::from("/usr/local/share/icons"));
            bases.push(PathBuf::from("/usr/share/icons"));
        }
        Category::Cursors => {
            bases.push(PathBuf::from("/usr/local/share/icons"));
            bases.push(PathBuf::from("/usr/share/icons"));
        }
    }

    bases.sort();
    bases.dedup();
    bases
}

fn current_theme(category: Category) -> Option<String> {
    let output = std::process::Command::new("gsettings")
        .args(["get", category.settings_schema(), category.settings_key()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some(value.trim_matches('\'').to_string())
}

fn apply_theme(category: Category, name: &str) -> Result<(), String> {
    let schema = category.settings_schema();
    let key = category.settings_key();
    let schema_check = std::process::Command::new("gsettings")
        .args(["writable", schema, key])
        .output()
        .map_err(|error| format!("Could not run gsettings: {error}"))?;

    if !schema_check.status.success()
        || String::from_utf8_lossy(&schema_check.stdout).trim() != "true"
    {
        return Err(match category {
            Category::Shell => "The GNOME User Themes extension must be installed and enabled to apply shell themes.".to_string(),
            _ => format!("The GNOME setting {schema} {key} is not available or writable."),
        });
    }

    let result = std::process::Command::new("gsettings")
        .args(["set", schema, key, name])
        .output()
        .map_err(|error| format!("Could not run gsettings: {error}"))?;

    if result.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Could not apply ‘{name}’: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ))
    }
}
