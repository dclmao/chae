use adw::prelude::*;
use std::ffi::{c_char, c_int, c_uint, CString};
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
        .application_id("io.github.dclmao.Chae")
        .build();
    gtk::Window::set_default_icon_name("io.github.dclmao.Chae");
    if let Some(display) = gtk::gdk::Display::default() {
        let icon_theme = gtk::IconTheme::for_display(&display);
        icon_theme.add_search_path(concat!(env!("CARGO_MANIFEST_DIR"), "/data/icons"));
    }

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
    window.set_size_request(760, 620);

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
            .application_icon("io.github.dclmao.Chae")
            .version(env!("CARGO_PKG_VERSION"))
            .developer_name("dclmao")
            .license_type(gtk::License::Gpl30)
            .comments("A local GNOME appearance manager for themes, icons, and cursors.")
            .website("http://github.com/dclmao/chae")
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
    list.set_selection_mode(gtk::SelectionMode::Single);

    let shell_gallery = gtk::FlowBox::new();
    shell_gallery.set_selection_mode(gtk::SelectionMode::Single);
    shell_gallery.set_homogeneous(false);
    shell_gallery.set_min_children_per_line(1);
    shell_gallery.set_max_children_per_line(4);
    shell_gallery.set_row_spacing(14);
    shell_gallery.set_column_spacing(14);
    shell_gallery.set_activate_on_single_click(true);
    shell_gallery.set_visible(false);

    let status = gtk::Label::builder().xalign(0.0).wrap(true).build();
    status.add_css_class("dim-label");

    let intro_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    intro_row.set_valign(gtk::Align::Center);
    let intro_labels = gtk::Box::new(gtk::Orientation::Vertical, 4);
    intro_labels.set_hexpand(true);
    intro_labels.append(&intro);
    intro_labels.append(&description);
    intro_row.append(&intro_labels);

    let apply_button = gtk::Button::with_label("Apply");
    apply_button.add_css_class("suggested-action");
    apply_button.set_size_request(112, 42);
    apply_button.set_valign(gtk::Align::Center);
    apply_button.set_sensitive(false);
    intro_row.append(&apply_button);

    content_box.append(&intro_row);
    content_box.append(&shell_gallery);
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
    category_stack.set_visible_child_name(Category::Shell.settings_key());

    let category_stack_page = adw::NavigationPage::new(&content_page, "Appearance");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.append(&header);
    root.set_vexpand(true);
    root.append(&category_stack_page);
    window.set_content(Some(&root));

    let selected_theme: std::rc::Rc<std::cell::RefCell<Option<String>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));
    let icon_preview_rows: std::rc::Rc<std::cell::RefCell<Vec<(gtk::Box, gtk::Box)>>> =
        std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let icon_preview_rows_for_resize = icon_preview_rows.clone();
    let last_window_width = std::rc::Rc::new(std::cell::Cell::new(0));
    let last_window_width_for_resize = last_window_width.clone();
    window.add_tick_callback(move |window, _| {
        let width = window.width();
        if last_window_width_for_resize.replace(width) != width {
            let visible_count = (6 + ((width - 760).max(0) / 46) as usize).min(PREVIEW_ICONS.len());
            for (content, previews) in icon_preview_rows_for_resize.borrow().iter() {
                let title_width = content.width();
                let row_visible_count =
                    (6 + ((title_width - 700).max(0) / 46) as usize).min(visible_count);
                set_preview_icon_visibility(previews, row_visible_count);
            }
        }
        glib::ControlFlow::Continue
    });

    let selected_for_rows = selected_theme.clone();
    let apply_for_rows = apply_button.clone();
    list.connect_row_selected(move |_, row| {
        let selected = row.map(|row| row.widget_name().to_string());
        *selected_for_rows.borrow_mut() = selected;
        apply_for_rows.set_sensitive(selected_for_rows.borrow().is_some());
    });

    let selected_for_cards = selected_theme.clone();
    let apply_for_cards = apply_button.clone();
    shell_gallery.connect_selected_children_changed(move |gallery| {
        let selected = gallery
            .selected_children()
            .first()
            .map(|child| child.widget_name().to_string());
        *selected_for_cards.borrow_mut() = selected;
        apply_for_cards.set_sensitive(selected_for_cards.borrow().is_some());
    });

    let selected_for_apply = selected_theme.clone();
    let list_for_apply = list.clone();
    let status_for_apply = status.clone();
    let stack_for_apply = category_stack.clone();
    let apply_button_for_click = apply_button.clone();
    apply_button.connect_clicked(move |_| {
        let Some(name) = selected_for_apply.borrow().clone() else {
            return;
        };
        let category = match stack_for_apply.visible_child_name().as_deref() {
            Some("icon-theme") => Category::Icons,
            Some("cursor-theme") => Category::Cursors,
            _ => Category::Shell,
        };
        match apply_theme(category, &name) {
            Ok(()) => {
                if category == Category::Shell {
                    let current = current_theme(Category::Shell).unwrap_or_else(|| "unknown".into());
                    status_for_apply.set_text(&format!(
                        "Shell theme setting is now ‘{current}’. Reload GNOME Shell or sign out and back in if the appearance has not updated."
                    ));
                } else {
                    status_for_apply.set_text(&format!("Applied ‘{name}’."));
                    populate_list(category, &list_for_apply, &status_for_apply);
                    selected_for_apply.borrow_mut().take();
                    apply_button_for_click.set_sensitive(false);
                }
            }
            Err(error) => status_for_apply.set_text(&error),
        }
    });

    let list_for_selection = list.clone();
    let shell_gallery_for_selection = shell_gallery.clone();
    let status_for_selection = status.clone();
    let selected_for_selection = selected_theme.clone();
    let apply_for_selection = apply_button.clone();
    category_stack.connect_visible_child_notify(move |stack| {
        let category = match stack.visible_child_name().as_deref() {
            Some("icon-theme") => Category::Icons,
            Some("cursor-theme") => Category::Cursors,
            _ => Category::Shell,
        };
        selected_for_selection.borrow_mut().take();
        apply_for_selection.set_sensitive(false);
        let show_shell_gallery = category == Category::Shell;
        shell_gallery_for_selection.set_visible(show_shell_gallery);
        list_for_selection.set_visible(!show_shell_gallery);
        if category == Category::Shell {
            populate_shell_gallery(&shell_gallery_for_selection, &status_for_selection);
        } else if category == Category::Icons {
            show_icon_loading(&list_for_selection, &status_for_selection);
            let list = list_for_selection.clone();
            let status = status_for_selection.clone();
            let icon_rows = icon_preview_rows.clone();
            glib::MainContext::default().spawn_local(async move {
                glib::timeout_future(std::time::Duration::from_millis(50)).await;
                let mut themes = discover(Category::Icons);
                themes.retain(|path| {
                    path.file_name()
                        .is_some_and(|name| !name.to_string_lossy().eq_ignore_ascii_case("hicolor"))
                });
                themes.sort_by_key(|path| {
                    path.file_name()
                        .map(|name| name.to_string_lossy().to_lowercase())
                });
                let total = themes.len();
                icon_rows.borrow_mut().clear();
                let mut prepared_rows = Vec::new();
                for theme_path in themes {
                    if let Some((row, content, previews)) = prepare_icon_theme_row(&theme_path) {
                        prepared_rows.push((row, content, previews));
                    }
                    glib::timeout_future(std::time::Duration::from_millis(1)).await;
                }
                while let Some(child) = list.first_child() {
                    list.remove(&child);
                }
                for (row, content, previews) in prepared_rows {
                    list.append(&row);
                    icon_rows.borrow_mut().push((content, previews));
                }
                status.set_text(&format!(
                    "{} additional themes found. Directories checked: {}",
                    total,
                    search_directories(Category::Icons)
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            });
        } else {
            shell_gallery_for_selection.set_visible(false);
            list_for_selection.set_visible(true);
            populate_list(category, &list_for_selection, &status_for_selection);
        }
    });
    shell_gallery.set_visible(true);
    list.set_visible(false);
    populate_shell_gallery(&shell_gallery, &status);

    let list_for_refresh = list.clone();
    let shell_gallery_for_refresh = shell_gallery.clone();
    let status_for_refresh = status.clone();
    let selected_for_refresh = selected_theme.clone();
    let apply_for_refresh = apply_button.clone();
    let stack_for_refresh = category_stack.clone();
    refresh_button.connect_clicked(move |_| {
        let category = match stack_for_refresh.visible_child_name().as_deref() {
            Some("icon-theme") => Category::Icons,
            Some("cursor-theme") => Category::Cursors,
            _ => Category::Shell,
        };
        selected_for_refresh.borrow_mut().take();
        apply_for_refresh.set_sensitive(false);
        if category == Category::Shell {
            populate_shell_gallery(&shell_gallery_for_refresh, &status_for_refresh);
        } else {
            populate_list(category, &list_for_refresh, &status_for_refresh);
        }
    });

    window.present();
}

fn format_theme_name(name: &str) -> String {
    name.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut characters = word.chars();
            if let Some(first) = characters.next() {
                first.to_uppercase().collect::<String>() + characters.as_str()
            } else {
                String::new()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn show_icon_loading(list: &gtk::ListBox, status: &gtk::Label) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let loading = gtk::Box::new(gtk::Orientation::Vertical, 12);
    loading.set_halign(gtk::Align::Center);
    loading.set_valign(gtk::Align::Center);
    loading.set_margin_top(48);
    loading.set_margin_bottom(48);
    let spinner = gtk::Spinner::new();
    spinner.set_spinning(true);
    spinner.set_halign(gtk::Align::Center);
    let label = gtk::Label::new(Some("Loading application icon previews…"));
    label.add_css_class("dim-label");
    loading.append(&spinner);
    loading.append(&label);
    list.append(&loading);
    status.set_text("Preparing previews for all installed application icons…");
}

fn prepare_icon_theme_row(theme_path: &Path) -> Option<(gtk::ListBoxRow, gtk::Box, gtk::Box)> {
    let name = theme_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())?;
    let display_name = format_theme_name(&name);
    let current = current_theme(Category::Icons);
    let row = gtk::ListBoxRow::new();
    row.set_widget_name(&name);
    row.set_activatable(true);
    row.set_selectable(true);
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    content.set_margin_top(14);
    content.set_margin_bottom(14);
    content.set_margin_start(16);
    content.set_margin_end(16);
    let title = gtk::Label::builder()
        .label(&display_name)
        .xalign(0.0)
        .build();
    title.add_css_class("heading");
    title.set_hexpand(true);
    title.set_width_request(120);
    title.set_max_width_chars(28);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    if current.as_deref() == Some(name.as_str()) {
        title.set_tooltip_text(Some("Currently selected"));
    }
    content.append(&title);
    let previews = icon_preview_strip(theme_path);
    previews.set_hexpand(true);
    set_preview_icon_visibility(&previews, 6);
    content.append(&previews);
    row.set_child(Some(&content));
    Some((row, content, previews))
}

fn populate_shell_gallery(gallery: &gtk::FlowBox, status: &gtk::Label) {
    while let Some(child) = gallery.first_child() {
        gallery.remove(&child);
    }

    let mut themes = discover(Category::Shell);
    themes.sort_by_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
    });
    for path in &themes {
        if let Some(card) = shell_theme_card(path) {
            gallery.insert(&card, -1);
        }
    }
    status.set_text(&format!(
        "{} shell themes found. Directories checked: {}",
        themes.len(),
        search_directories(Category::Shell)
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));
}

fn shell_theme_card(theme_path: &Path) -> Option<gtk::FlowBoxChild> {
    let name = theme_path.file_name()?.to_string_lossy().into_owned();
    let display_name = format_theme_name(&name);
    let colors = shell_theme_colors(theme_path);

    let flow_child = gtk::FlowBoxChild::new();
    flow_child.set_widget_name(&name);

    let card = gtk::Box::new(gtk::Orientation::Vertical, 10);
    card.add_css_class("card");
    card.add_css_class("shell-theme-card");
    let theme_css_class = format!(
        "shell-theme-{}",
        name.chars()
            .map(|character| if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            })
            .collect::<String>()
    );
    card.add_css_class(&theme_css_class);
    card.set_size_request(280, 190);
    card.set_valign(gtk::Align::Start);
    card.set_margin_top(8);
    card.set_margin_bottom(8);
    card.set_margin_start(8);
    card.set_margin_end(8);
    card.set_focusable(true);
    card.set_cursor_from_name(Some("pointer"));

    let title = gtk::Label::builder()
        .label(&display_name)
        .xalign(0.5)
        .justify(gtk::Justification::Center)
        .build();
    title.add_css_class("heading");
    title.set_margin_top(12);
    title.set_margin_start(12);
    title.set_margin_end(12);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    card.append(&title);

    let preview = gtk::Box::new(gtk::Orientation::Vertical, 0);
    preview.set_margin_start(12);
    preview.set_margin_end(12);
    preview.set_margin_bottom(12);
    preview.add_css_class("theme-preview");
    preview.set_overflow(gtk::Overflow::Hidden);
    preview.set_size_request(0, 120);
    preview.add_css_class("quick-settings-preview");

    let color_blocks = gtk::Grid::new();
    color_blocks.set_column_spacing(6);
    color_blocks.set_row_spacing(6);
    color_blocks.set_margin_top(8);
    color_blocks.set_margin_start(8);
    color_blocks.set_margin_end(8);
    color_blocks.set_margin_bottom(8);
    color_blocks.set_valign(gtk::Align::Start);
    color_blocks.set_hexpand(true);
    for (index, (color, _)) in colors.iter().enumerate() {
        let block = gtk::Overlay::new();
        block.set_size_request(0, 38);
        block.set_hexpand(true);
        block.set_tooltip_text(Some(color));
        let drawing = gtk::DrawingArea::new();
        let rgba = gtk::gdk::RGBA::parse(color).ok()?;
        drawing.set_draw_func(move |_, context, width, height| {
            context.set_source_rgba(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                rgba.alpha() as f64,
            );
            context.rectangle(0.0, 0.0, width as f64, height as f64);
            let _ = context.fill();
        });
        block.set_child(Some(&drawing));

        color_blocks.attach(&block, (index % 2) as i32, (index / 2) as i32, 1, 1);
    }
    preview.append(&color_blocks);
    card.append(&preview);

    let click = gtk::GestureClick::new();
    let child_for_click = flow_child.clone();
    click.connect_released(move |_, _, _, _| {
        if let Some(gallery) = child_for_click.parent().and_downcast::<gtk::FlowBox>() {
            gallery.select_child(&child_for_click);
        }
    });
    card.add_controller(click);
    flow_child.set_child(Some(&card));
    Some(flow_child)
}

fn shell_theme_colors(theme_path: &Path) -> Vec<(String, String)> {
    let mut colors: Vec<(String, String, usize)> = Vec::new();
    let mut stylesheets = Vec::new();
    let shell_stylesheet = theme_path.join("gnome-shell/gnome-shell.css");
    if shell_stylesheet.is_file() {
        stylesheets.push((shell_stylesheet, 3));
    }
    for gtk_dir in ["gtk-4.0", "gtk-3.0"] {
        let gtk_path = theme_path.join(gtk_dir);
        for filename in ["gtk.css", "gtk-dark.css"] {
            let stylesheet = gtk_path.join(filename);
            if stylesheet.is_file() {
                stylesheets.push((stylesheet, 2));
            }
        }
    }

    if stylesheets.is_empty() {
        stylesheets.extend(installed_gtk_stylesheets(theme_path));
    }
    if theme_path.file_name().is_some_and(|name| {
        matches!(
            name.to_string_lossy().to_ascii_lowercase().as_str(),
            "adwaita" | "adwaita-dark" | "highcontrast"
        )
    }) {
        stylesheets.extend(gnome_shell_resource_stylesheets(theme_path));
    }
    let resource_variant = theme_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| match name.to_ascii_lowercase().as_str() {
            "adwaita-dark" => "dark",
            "highcontrast" => "high-contrast",
            _ => "light",
        });
    for (stylesheet, source_weight) in stylesheets {
        let Ok(css) = std::fs::read_to_string(&stylesheet) else {
            continue;
        };
        collect_stylesheet_colors(&css, source_weight, resource_variant, &mut colors);
    }
    if !colors
        .iter()
        .any(|(color, _, _)| PaletteFamily::Blue.matches(color))
    {
        for (stylesheet, source_weight) in builtin_gtk_stylesheets() {
            let Ok(css) = std::fs::read_to_string(stylesheet) else {
                continue;
            };
            collect_stylesheet_colors(&css, source_weight, None, &mut colors);
        }
    }
    if !colors
        .iter()
        .any(|(color, _, _)| PaletteFamily::Blue.matches(color))
    {
        for (stylesheet, source_weight) in installed_gtk_stylesheets(theme_path) {
            let Ok(css) = std::fs::read_to_string(stylesheet) else {
                continue;
            };
            collect_stylesheet_colors(&css, source_weight, None, &mut colors);
        }
    }
    if !colors
        .iter()
        .any(|(color, _, _)| PaletteFamily::Blue.matches(color))
    {
        colors.extend(runtime_gtk_theme_colors(theme_path));
    }
    let mut palette = Vec::new();
    for family in [
        PaletteFamily::White,
        PaletteFamily::Black,
        PaletteFamily::Blue,
        PaletteFamily::Green,
        PaletteFamily::Red,
        PaletteFamily::Yellow,
        PaletteFamily::Pink,
        PaletteFamily::Gray,
    ] {
        if let Some((color, raw, _)) = colors
            .iter()
            .filter(|(color, _, _)| family.matches(color))
            .max_by_key(|(_, _, count)| count)
        {
            palette.push((color.clone(), raw.clone()));
        }
    }
    palette
}

fn runtime_gtk_theme_colors(theme_path: &Path) -> Vec<(String, String, usize)> {
    let Some(theme_name) = theme_path.file_name().and_then(|name| name.to_str()) else {
        return Vec::new();
    };
    let Some(settings) = gtk::Settings::default() else {
        return Vec::new();
    };
    let previous_theme = settings.gtk_theme_name();
    let candidates = match theme_name.to_ascii_lowercase().as_str() {
        "adwaita-dark" => vec!["Adwaita-dark", "Adwaita"],
        "adwaita" => vec!["Adwaita", "Adwaita-dark"],
        "highcontrast" => vec!["HighContrast"],
        _ => vec![theme_name],
    };
    let mut sampled = Vec::new();
    for candidate in candidates {
        settings.set_gtk_theme_name(Some(candidate));
        while gtk::glib::MainContext::default().pending() {
            gtk::glib::MainContext::default().iteration(false);
        }
        sampled = sample_gtk_widget_colors();
        if !sampled.is_empty() {
            break;
        }
    }
    if let Some(previous_theme) = previous_theme {
        settings.set_gtk_theme_name(Some(&previous_theme));
        while gtk::glib::MainContext::default().pending() {
            gtk::glib::MainContext::default().iteration(false);
        }
    }
    sampled
}

fn sample_gtk_widget_colors() -> Vec<(String, String, usize)> {
    let widgets: Vec<gtk::Widget> = vec![
        gtk::Window::new().upcast(),
        gtk::Button::with_label("Theme sample").upcast(),
        gtk::Entry::new().upcast(),
        gtk::TextView::new().upcast(),
        gtk::ListBox::new().upcast(),
        gtk::Switch::new().upcast(),
        gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0).upcast(),
        gtk::CheckButton::with_label("Selected").upcast(),
    ];
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        ".theme-palette-sample { color: @theme_fg_color; background-color: @theme_bg_color; }\
         .theme-palette-sample:checked, .theme-palette-sample:selected, .theme-palette-sample slider { color: @theme_selected_fg_color; background-color: @theme_selected_bg_color; }\
         .theme-palette-sample button, .theme-palette-sample switch:checked { background-color: @accent_bg_color; color: @accent_fg_color; }",
    );
    let Some(display) = gtk::gdk::Display::default() else {
        return Vec::new();
    };
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    let mut colors: Vec<(String, String, usize)> = Vec::new();
    for (index, widget) in widgets.into_iter().enumerate() {
        widget.add_css_class("theme-palette-sample");
        for state in [
            gtk::StateFlags::NORMAL,
            gtk::StateFlags::ACTIVE,
            gtk::StateFlags::SELECTED,
        ] {
            widget.set_state_flags(state, true);
            let context = widget.style_context();
            context.save();
            let background = context
                .lookup_color("theme_selected_bg_color")
                .or_else(|| context.lookup_color("accent_bg_color"))
                .or_else(|| context.lookup_color("theme_bg_color"))
                .or_else(|| context.lookup_color("window_bg_color"));
            let foreground = context
                .lookup_color("theme_selected_fg_color")
                .or_else(|| context.lookup_color("accent_fg_color"))
                .or_else(|| context.lookup_color("theme_fg_color"))
                .or_else(|| context.lookup_color("window_fg_color"));
            context.restore();
            let context_color = context.color();
            for color in [background, foreground, Some(context_color)]
                .into_iter()
                .flatten()
            {
                let value = format!(
                    "rgba({}, {}, {}, {:.3})",
                    (color.red() * 255.0).round() as u8,
                    (color.green() * 255.0).round() as u8,
                    (color.blue() * 255.0).round() as u8,
                    color.alpha()
                );
                if let Some((_, _, count)) = colors
                    .iter_mut()
                    .find(|(existing, _, _)| same_css_color(existing, &value))
                {
                    *count += 1;
                } else {
                    colors.push((value.clone(), value, 1 + usize::from(index == 0)));
                }
            }
        }
    }
    gtk::style_context_remove_provider_for_display(&display, &provider);
    colors
}

fn builtin_gtk_stylesheets() -> Vec<(PathBuf, usize)> {
    let mut stylesheets = Vec::new();
    for root in ["/usr/share/gtk-4.0", "/usr/share/gtk-3.0"] {
        for path in [
            PathBuf::from(root).join("theme/Default/Default.css"),
            PathBuf::from(root).join("theme/Default/Default-dark.css"),
            PathBuf::from(root).join("theme/HighContrast/HighContrast.css"),
            PathBuf::from(root).join("theme/HighContrast/HighContrast-dark.css"),
        ] {
            if path.is_file() && !stylesheets.iter().any(|(existing, _)| existing == &path) {
                stylesheets.push((path, 1));
            }
        }
    }
    stylesheets
}

fn installed_gtk_stylesheets(theme_path: &Path) -> Vec<(PathBuf, usize)> {
    let Some(theme_name) = theme_path.file_name().and_then(|name| name.to_str()) else {
        return Vec::new();
    };
    let mut roots = vec![
        PathBuf::from("/usr/share/themes"),
        PathBuf::from("/usr/local/share/themes"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        roots.push(home.join(".themes"));
        roots.push(home.join(".local/share/themes"));
    }
    let mut stylesheets = Vec::new();
    let candidates = match theme_name.to_ascii_lowercase().as_str() {
        "adwaita" => vec!["Adwaita".to_string(), "Adwaita-dark".to_string()],
        "highcontrast" => vec!["HighContrast".to_string()],
        _ => vec![theme_name.to_string(), format!("{theme_name}-dark")],
    };
    for root in roots {
        for name in &candidates {
            for gtk_dir in ["gtk-4.0", "gtk-3.0"] {
                for filename in ["gtk.css", "gtk-dark.css"] {
                    let stylesheet = root.join(name).join(gtk_dir).join(filename);
                    if stylesheet.is_file()
                        && !stylesheets
                            .iter()
                            .any(|(existing, _)| existing == &stylesheet)
                    {
                        stylesheets.push((stylesheet, 1));
                    }
                }
            }
        }
    }
    stylesheets
}

fn gnome_shell_resource_stylesheets(theme_path: &Path) -> Vec<(PathBuf, usize)> {
    let Some(theme_name) = theme_path.file_name().and_then(|name| name.to_str()) else {
        return Vec::new();
    };
    let resources = [
        PathBuf::from("/usr/share/gnome-shell/gnome-shell-theme.gresource"),
        PathBuf::from("/usr/local/share/gnome-shell/gnome-shell-theme.gresource"),
    ];
    let variant = match theme_name.to_ascii_lowercase().as_str() {
        "adwaita-dark" => "dark",
        "highcontrast" => "high-contrast",
        _ => "light",
    };
    let resource_path = format!("/org/gnome/shell/theme/gnome-shell-{variant}.css");
    let Some(resource_file) = resources.iter().find(|resource| resource.is_file()) else {
        return Vec::new();
    };
    let Ok(resource) = gio::Resource::load(resource_file) else {
        return Vec::new();
    };
    let Ok(data) = resource.lookup_data(&resource_path, gio::ResourceLookupFlags::NONE) else {
        return Vec::new();
    };
    let Ok(css) = std::str::from_utf8(data.as_ref()) else {
        return Vec::new();
    };
    let temp_path = std::env::temp_dir().join(format!("chae-shell-theme-{variant}.css"));
    if std::fs::write(&temp_path, css).is_err() {
        return Vec::new();
    }
    vec![(temp_path, 4)]
}

fn resolve_shell_color(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if let Some(color) = normalize_css_color(trimmed) {
        return Some(color);
    }
    if let Some(inner) = trimmed
        .strip_prefix("st-mix(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let args = split_css_args(inner);
        if args.len() == 3 {
            let first = resolve_shell_color(args[0])?;
            let second = resolve_shell_color(args[1])?;
            let percentage = args[2].trim_end_matches('%').parse::<f32>().ok()? / 100.0;
            return Some(mix_css_colors(&first, &second, percentage));
        }
    }
    None
}

fn split_css_args(value: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth = 0_i32;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                args.push(value.get(start..index).unwrap_or_default().trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    args.push(value.get(start..).unwrap_or_default().trim());
    args
}

fn mix_css_colors(first: &str, second: &str, amount: f32) -> String {
    let first = gtk::gdk::RGBA::parse(first).expect("resolved CSS color");
    let second = gtk::gdk::RGBA::parse(second).expect("resolved CSS color");
    normalize_css_color(&format!(
        "rgba({}, {}, {}, {})",
        ((first.red() * (1.0 - amount) + second.red() * amount) * 255.0).round() as u8,
        ((first.green() * (1.0 - amount) + second.green() * amount) * 255.0).round() as u8,
        ((first.blue() * (1.0 - amount) + second.blue() * amount) * 255.0).round() as u8,
        first.alpha() * (1.0 - amount) + second.alpha() * amount
    ))
    .unwrap_or_else(|| first.to_string())
}

fn collect_stylesheet_colors(
    css: &str,
    source_weight: usize,
    resource_variant: Option<&str>,
    colors: &mut Vec<(String, String, usize)>,
) {
    let mut definitions = std::collections::HashMap::new();
    for line in css.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("@define-color ") {
            if let Some((name, value)) = rest.trim_end_matches(';').split_once(char::is_whitespace)
            {
                definitions.insert(name.to_string(), value.trim().to_string());
            }
        }
    }
    for rule in css.split('}') {
        let Some((_, declarations)) = rule.split_once('{') else {
            continue;
        };
        for declaration in declarations.split(';') {
            let Some((key, value)) = declaration.split_once(':') else {
                continue;
            };
            let property = key.trim().to_lowercase();
            if !matches!(
                property.as_str(),
                "background"
                    | "background-color"
                    | "color"
                    | "border-color"
                    | "outline-color"
                    | "selection-background-color"
                    | "selected-color"
                    | "warning-color"
                    | "error-color"
                    | "success-color"
                    | "-barlevel-active-background-color"
                    | "-barlevel-background-color"
                    | "-barlevel-overdrive-color"
                    | "-slider-handle-border-color"
                    | "-pie-border-color"
                    | "-pie-background-color"
            ) {
                continue;
            }
            let value = value.split('!').next().unwrap_or(value).trim();
            let resolved = value
                .strip_prefix('@')
                .and_then(|name| definitions.get(name))
                .map(String::as_str)
                .unwrap_or(value);
            let resolved = match resolved {
                "-st-accent-color" => match resource_variant {
                    Some("dark") => "#3584e4",
                    Some("high-contrast") => "#1c71d8",
                    _ => "#3584e4",
                },
                "-st-accent-fg-color" => "#ffffff",
                _ => resolved,
            };
            let resolved = resolve_shell_color(resolved).unwrap_or_else(|| resolved.to_string());
            let Some(color) = normalize_css_color(&resolved) else {
                continue;
            };
            if let Some((_, _, count)) = colors
                .iter_mut()
                .find(|(existing, _, _)| same_css_color(existing, &color))
            {
                *count += source_weight;
            } else {
                colors.push((color, value.to_string(), source_weight));
            }
        }
    }
}

enum PaletteFamily {
    White,
    Black,
    Blue,
    Green,
    Red,
    Yellow,
    Pink,
    Gray,
}

impl PaletteFamily {
    fn matches(&self, color: &str) -> bool {
        let Ok(rgba) = gtk::gdk::RGBA::parse(color) else {
            return false;
        };
        let red = rgba.red();
        let green = rgba.green();
        let blue = rgba.blue();
        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let chroma = max - min;
        match self {
            Self::White => min > 0.78 && chroma < 0.12,
            Self::Black => max < 0.18,
            Self::Gray => chroma < 0.10 && (0.18..=0.78).contains(&max),
            Self::Red => red > green * 1.35 && red > blue * 1.25 && red > 0.30,
            Self::Yellow => red > 0.45 && green > 0.35 && blue < red * 0.65 && blue < green * 0.75,
            Self::Green => green > red * 1.15 && green > blue * 1.10 && green > 0.22,
            Self::Blue => blue > red * 1.15 && blue > green * 1.08 && blue > 0.25,
            Self::Pink => red > 0.38 && blue > 0.28 && green < red * 0.90 && green < blue * 0.95,
        }
    }
}

fn same_css_color(left: &str, right: &str) -> bool {
    let (Some(left), Some(right)) = (
        gtk::gdk::RGBA::parse(left).ok(),
        gtk::gdk::RGBA::parse(right).ok(),
    ) else {
        return false;
    };
    (left.red() - right.red()).abs() < 0.002
        && (left.green() - right.green()).abs() < 0.002
        && (left.blue() - right.blue()).abs() < 0.002
        && (left.alpha() - right.alpha()).abs() < 0.002
}

fn normalize_css_color(value: &str) -> Option<String> {
    let rgba = gtk::gdk::RGBA::parse(value.trim()).ok()?;
    Some(format!(
        "rgba({}, {}, {}, {:.3})",
        (rgba.red() * 255.0).round() as u8,
        (rgba.green() * 255.0).round() as u8,
        (rgba.blue() * 255.0).round() as u8,
        rgba.alpha()
    ))
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
    let default_row = gtk::ListBoxRow::new();
    default_row.set_widget_name(default_name);
    default_row.set_activatable(true);
    default_row.set_selectable(true);
    let default_content = gtk::Box::new(gtk::Orientation::Vertical, 3);
    default_content.set_margin_top(14);
    default_content.set_margin_bottom(14);
    default_content.set_margin_start(16);
    default_content.set_margin_end(16);
    let default_title = gtk::Label::builder().label("Default").xalign(0.0).build();
    default_title.add_css_class("heading");
    default_content.append(&default_title);
    let default_subtitle = gtk::Label::builder()
        .label(if current.as_deref() == Some(default_name) {
            "Currently selected · Adwaita"
        } else {
            "Use the GNOME default appearance"
        })
        .xalign(0.0)
        .build();
    default_subtitle.add_css_class("dim-label");
    default_content.append(&default_subtitle);
    default_row.set_child(Some(&default_content));
    list.append(&default_row);

    for path in &themes {
        let Some(name) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            continue;
        };
        let display_name = format_theme_name(&name);
        let list_row = gtk::ListBoxRow::new();
        list_row.set_widget_name(&name);
        list_row.set_activatable(true);
        list_row.set_selectable(true);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        row.set_margin_top(14);
        row.set_margin_bottom(14);
        row.set_margin_start(16);
        row.set_margin_end(16);

        let title = gtk::Label::builder()
            .label(&display_name)
            .xalign(0.0)
            .build();
        title.add_css_class("heading");
        title.set_hexpand(true);
        title.set_width_request(120);
        title.set_max_width_chars(28);
        title.set_ellipsize(gtk::pango::EllipsizeMode::End);
        row.append(&title);
        if current.as_deref() == Some(name.as_str()) {
            title.set_tooltip_text(Some("Currently selected"));
        }
        match category {
            Category::Icons => row.append(&icon_preview_strip(&path)),
            Category::Cursors => {
                let previews = cursor_preview_strip(&path);
                previews.set_halign(gtk::Align::End);
                previews.set_hexpand(true);
                row.append(&previews);
            }
            Category::Shell => {}
        }
        list_row.set_child(Some(&row));
        list.append(&list_row);
    }

    if themes.is_empty() {
        let row = adw::ActionRow::builder()
            .title("No additional themes found")
            .subtitle("Only the default appearance is available.")
            .build();
        row.set_selectable(false);
        row.set_activatable(false);
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

const PREVIEW_ICONS: [(&str, &str, &str); 18] = [
    ("Firefox", "firefox", "firefox-esr"),
    ("Folders", "folder", "folder"),
    ("Settings", "preferences-system", "org.gnome.Settings"),
    ("Files", "org.gnome.Nautilus", "system-file-manager"),
    ("Steam", "steam", "steam"),
    ("Terminal", "utilities-terminal", "org.gnome.Terminal"),
    ("VLC", "vlc", "vlc"),
    ("MPV", "mpv", "mpv"),
    ("Thunderbird", "thunderbird", "thunderbird"),
    ("Kdenlive", "kdenlive", "kdenlive"),
    ("OBS", "com.obsproject.Studio", "obs"),
    ("LibreWolf", "librewolf", "librewolf"),
    ("Signal", "signal-desktop", "signal"),
    ("Stoat", "stoat", "stoat"),
    ("Vesktop", "vesktop", "vesktop"),
    ("Ghostty", "com.mitchellh.ghostty", "ghostty"),
    ("Kitty", "kitty", "kitty"),
    ("Lutris", "net.lutris.Lutris", "lutris"),
];

fn icon_preview_strip(theme_path: &Path) -> gtk::Box {
    let strip = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    strip.set_halign(gtk::Align::End);

    let icon_theme = gtk::IconTheme::new();
    let parent_path = theme_path.parent().unwrap_or(theme_path);
    let search_path: [&Path; 1] = [parent_path];
    icon_theme.set_search_path(&search_path[..]);
    icon_theme.set_theme_name(theme_path.file_name().and_then(|name| name.to_str()));

    for (label, first_candidate, second_candidate) in PREVIEW_ICONS {
        let image = gtk::Image::new();
        image.set_pixel_size(36);
        image.set_tooltip_text(Some(label));
        let mut icon = icon_theme.lookup_icon(
            first_candidate.trim(),
            &[],
            36,
            1,
            gtk::TextDirection::Ltr,
            gtk::IconLookupFlags::FORCE_REGULAR,
        );
        if icon.file().is_none() {
            icon = icon_theme.lookup_icon(
                second_candidate,
                &[],
                36,
                1,
                gtk::TextDirection::Ltr,
                gtk::IconLookupFlags::FORCE_REGULAR,
            );
        }
        image.set_paintable(Some(&icon));
        strip.append(&image);
    }
    strip
}

fn set_preview_icon_visibility(strip: &gtk::Box, count: usize) {
    let mut child = strip.first_child();
    for index in 0..PREVIEW_ICONS.len() {
        let Some(widget) = child else {
            break;
        };
        widget.set_visible(index < count);
        child = widget.next_sibling();
    }
}

#[repr(C)]
struct XcursorImage {
    version: c_uint,
    size: c_uint,
    width: c_uint,
    height: c_uint,
    xhot: c_uint,
    yhot: c_uint,
    delay: c_uint,
    pixels: *mut u32,
}

#[link(name = "Xcursor")]
unsafe extern "C" {
    fn XcursorFilenameLoadAllImages(filename: *const c_char) -> *mut XcursorImages;
    fn XcursorImagesDestroy(images: *mut XcursorImages);
}

#[repr(C)]
struct XcursorImages {
    nimage: c_int,
    images: *mut *mut XcursorImage,
    name: *mut c_char,
}

fn cursor_pixbuf(theme_path: &Path, cursor_name: &str) -> Option<gtk::gdk_pixbuf::Pixbuf> {
    let cursor_file = CString::new(theme_path.join("cursors").join(cursor_name).to_str()?).ok()?;
    let loaded = unsafe { XcursorFilenameLoadAllImages(cursor_file.as_ptr()) };
    if loaded.is_null() {
        return None;
    }
    let images = unsafe { &*loaded };
    if images.nimage <= 0 || images.images.is_null() {
        unsafe { XcursorImagesDestroy(loaded) };
        return None;
    }
    let mut chosen: *mut XcursorImage = std::ptr::null_mut();
    let mut chosen_size = 0_u32;
    for index in 0..images.nimage as usize {
        let candidate = unsafe { *images.images.add(index) };
        if candidate.is_null() {
            continue;
        }
        let image = unsafe { &*candidate };
        if image.size > chosen_size {
            chosen = candidate;
            chosen_size = image.size;
        }
    }
    if chosen.is_null() {
        unsafe { XcursorImagesDestroy(loaded) };
        return None;
    }
    let image = unsafe { &*chosen };
    let width = image.width as i32;
    let height = image.height as i32;
    let pixbuf =
        gtk::gdk_pixbuf::Pixbuf::new(gtk::gdk_pixbuf::Colorspace::Rgb, true, 8, width, height);
    if let Some(ref pixbuf) = pixbuf {
        let rowstride = pixbuf.rowstride() as usize;
        let channels = pixbuf.n_channels() as usize;
        let pixels = unsafe { pixbuf.pixels() };
        for y in 0..height as usize {
            for x in 0..width as usize {
                let source = unsafe { *image.pixels.add(y * width as usize + x) } as u32;
                let dest = y * rowstride + x * channels;
                pixels[dest] = ((source >> 16) & 0xff) as u8;
                pixels[dest + 1] = ((source >> 8) & 0xff) as u8;
                pixels[dest + 2] = (source & 0xff) as u8;
                pixels[dest + 3] = ((source >> 24) & 0xff) as u8;
            }
        }
    }
    unsafe { XcursorImagesDestroy(loaded) };
    pixbuf
}

fn cursor_preview_strip(theme_path: &Path) -> gtk::Box {
    const CURSORS: [(&str, &str); 5] = [
        ("Default", "default"),
        ("Link", "pointer"),
        ("Resize", "col-resize"),
        ("Blocked", "not-allowed"),
        ("Zoom", "zoom-in"),
    ];

    let strip = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    strip.set_halign(gtk::Align::End);
    for (label, cursor_name) in CURSORS {
        let item = gtk::Box::new(gtk::Orientation::Vertical, 4);
        item.set_width_request(66);
        item.set_halign(gtk::Align::Center);

        let preview = gtk::Image::new();
        preview.set_pixel_size(48);
        preview.set_size_request(52, 48);
        if let Some(pixbuf) = cursor_pixbuf(theme_path, cursor_name) {
            preview.set_from_pixbuf(Some(&pixbuf));
            preview.set_tooltip_text(Some(&format!("{} cursor from this theme", label)));
        } else {
            preview.set_tooltip_text(Some(&format!("{} cursor is not available", label)));
        }
        item.append(&preview);

        let caption = gtk::Label::new(Some(label));
        caption.add_css_class("caption");
        caption.add_css_class("dim-label");
        item.append(&caption);
        strip.append(&item);
    }
    strip
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

    if !result.status.success() {
        return Err(format!(
            "Could not apply ‘{name}’: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    let applied = current_theme(category);
    if applied.as_deref() != Some(name) {
        return Err(format!(
            "The setting command succeeded, but GNOME reports the current theme as ‘{}’.",
            applied.as_deref().unwrap_or("unknown")
        ));
    }
    Ok(())
}
