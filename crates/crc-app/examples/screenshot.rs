use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use crc_app::Session;
use crc_theme::{Appearance, Density, Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::palette::{self, Action, PaletteView};
use crc_ui::view::settings::{BindingRow, Section, SettingsView, Toggle};
use crc_ui::view::welcome::{RecentEntry, WelcomeView};
use crc_ui::view::{self, CodeMetrics};
use crc_ui::{Offscreen, Shell, ShellState, TextRun};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;

fn main() -> anyhow::Result<()> {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let out = PathBuf::from("site/shots");
    std::fs::create_dir_all(&out)?;

    let mut session = Session::open(&root)?;
    for path in [
        "crates/crc-ui/src/view/logo.rs",
        "crates/crc-editor/src/document.rs",
        "crates/crc-theme/src/palette/dark.rs",
    ] {
        let _ = session.open_file(Path::new(path));
    }
    session.view.branch = "main".to_string();
    session.view.problems = 0;

    let mut canvas = Offscreen::new(WIDTH, HEIGHT)?;
    println!("rendering on {}", canvas.adapter());

    let full = ShellState::default();
    let stripped = ShellState {
        rail: false,
        minimap: false,
        panel: false,
        breadcrumbs: false,
        ..ShellState::default()
    };

    for (name, appearance, density, zen, query, state) in [
        (
            "dark",
            Appearance::Dark,
            Density::Balanced,
            false,
            None,
            &full,
        ),
        (
            "light",
            Appearance::Light,
            Density::Balanced,
            false,
            None,
            &full,
        ),
        (
            "zen",
            Appearance::Dark,
            Density::Balanced,
            true,
            None,
            &full,
        ),
        (
            "dense",
            Appearance::Dark,
            Density::Dense,
            false,
            None,
            &full,
        ),
        (
            "palette",
            Appearance::Dark,
            Density::Balanced,
            false,
            Some("тем"),
            &full,
        ),
        (
            "stripped",
            Appearance::Light,
            Density::Balanced,
            false,
            None,
            &stripped,
        ),
        (
            "welcome",
            Appearance::Dark,
            Density::Balanced,
            false,
            None,
            &full,
        ),
    ] {
        session.view.welcome = (name == "welcome").then(|| WelcomeView {
            recent: vec![
                RecentEntry {
                    name: "crc-code".into(),
                    path: "d:/Project/CRC Code".into(),
                    when: "12 мин назад".into(),
                },
                RecentEntry {
                    name: "minedres-legal".into(),
                    path: "d:/Project/minedress-legal".into(),
                    when: "вчера".into(),
                },
                RecentEntry {
                    name: "PrimalWorld".into(),
                    path: "d:/Project/PrimalWorld".into(),
                    when: "3 дн назад".into(),
                },
            ],
            hints: vec![
                (
                    "Ctrl+K".into(),
                    "Командная палитра — всё с клавиатуры".into(),
                ),
                ("Ctrl+O".into(), "Открыть другой проект".into()),
                ("Alt+Z".into(), "Zen — панели уходят, остаётся код".into()),
            ],
            hovered: None,
        });

        session.view.palette = query.map(|text| PaletteView {
            query: text.to_string(),
            rows: palette::filter(&actions(), text),
            selected: 0,
        });
        let mut theme = Theme::new(appearance).with_density(density);
        theme.zen = zen;

        let probe = TextRun::new(
            "0000000000",
            Rect::new(0.0, 0.0, 1000.0, 100.0),
            theme.type_scale.code,
            Rgba::hex(0x000000),
        )
        .mono();
        let (width, _) = canvas.measure(&probe);
        let metrics = CodeMetrics {
            char_width: width / 10.0,
            line_height: theme.type_scale.code * crc_theme::typography::LINE_HEIGHT_CODE,
        };

        let layout = Shell::compute(Rect::from_size(WIDTH as f32, HEIGHT as f32), &theme, state);
        let frame = view::draw(&layout, &theme, &session.view, metrics);
        let pixels = canvas.render_frame(&frame);

        let path = out.join(format!("{name}.png"));
        write_png(&path, WIDTH, HEIGHT, &pixels)?;
        println!("{}", path.display());
    }

    session.view.welcome = None;
    session.view.palette = None;

    for (name, section, capturing) in [
        ("settings", Section::Appearance, None),
        ("keys", Section::Keys, Some(3usize)),
    ] {
        let mut bindings = binding_rows();
        crc_ui::view::settings::mark_clashes(&mut bindings);

        session.view.settings = Some(SettingsView {
            section,
            query: String::new(),
            toggles: toggles(),
            bindings,
            capturing,
            hovered: None,
            scroll: 0,
        });

        let theme = Theme::new(Appearance::Dark).with_density(Density::Balanced);
        let probe = TextRun::new(
            "0000000000",
            Rect::new(0.0, 0.0, 1000.0, 100.0),
            theme.type_scale.code,
            Rgba::hex(0x000000),
        )
        .mono();
        let (width, _) = canvas.measure(&probe);
        let metrics = CodeMetrics {
            char_width: width / 10.0,
            line_height: theme.type_scale.code * crc_theme::typography::LINE_HEIGHT_CODE,
        };

        let layout = Shell::compute(
            Rect::from_size(WIDTH as f32, HEIGHT as f32),
            &theme,
            &ShellState::default(),
        );
        let frame = view::draw(&layout, &theme, &session.view, metrics);
        let pixels = canvas.render_frame(&frame);

        let path = out.join(format!("{name}.png"));
        write_png(&path, WIDTH, HEIGHT, &pixels)?;
        println!("{}", path.display());
    }

    Ok(())
}

fn toggles() -> Vec<Toggle> {
    vec![
        Toggle::new("dark", "Тёмная тема", "Светлый и тёмный набор цветов", true),
        Toggle::new(
            "rail",
            "Рейка действий",
            "Узкая полоса у левого края",
            true,
        ),
        Toggle::new("explorer", "Проводник", "Дерево файлов проекта", true),
        Toggle::new("tabs", "Вкладки", "Строка с открытыми файлами", true),
        Toggle::new(
            "breadcrumbs",
            "Путь над файлом",
            "Проект и имя файла",
            false,
        ),
        Toggle::new(
            "minimap",
            "Мини-карта",
            "Полоса обзора справа от кода",
            true,
        ),
        Toggle::new("panel", "Нижняя панель", "Терминал, проблемы, вывод", true),
    ]
}

fn binding_rows() -> Vec<BindingRow> {
    let row = |command: &str, title: &str, keys: &str, changed: bool| BindingRow {
        command: command.to_string(),
        title: title.to_string(),
        keys: keys.to_string(),
        clash: None,
        changed,
    };

    vec![
        row("save", "Сохранить файл", "Ctrl+S", false),
        row("close-tab", "Закрыть вкладку", "Ctrl+W", false),
        row("palette", "Командная палитра", "Ctrl+K", false),
        row("open-folder", "Открыть папку проекта", "Ctrl+O", false),
        row("settings", "Настройки", "Ctrl+,", false),
        row("theme", "Переключить светлую и тёмную тему", "Ctrl+D", true),
        row("sidebar", "Показать или скрыть проводник", "Ctrl+D", true),
        row("zen", "Zen — оставить только код", "Alt+Z", false),
        row("select-all", "Выделить всё", "Ctrl+A", false),
        row("undo", "Отменить правку", "Ctrl+Z", false),
        row("redo", "Вернуть правку", "", false),
    ]
}

fn actions() -> Vec<Action> {
    vec![
        Action::new("save", "Сохранить файл", "Файл").hint("Ctrl+S"),
        Action::new("close-tab", "Закрыть вкладку", "Файл").hint("Ctrl+W"),
        Action::new("undo", "Отменить правку", "Правка").hint("Ctrl+Z"),
        Action::new("select-all", "Выделить всё", "Правка").hint("Ctrl+A"),
        Action::new("theme", "Переключить светлую и тёмную тему", "Вид").hint("Ctrl+D"),
        Action::new("sidebar", "Показать или скрыть проводник", "Вид").hint("Ctrl+B"),
        Action::new("zen", "Zen — оставить только код", "Вид").hint("Alt+Z"),
        Action::new("dense", "Плотность: максимум мощи", "Вид").hint("Alt+3"),
    ]
}

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> anyhow::Result<()> {
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    encoder.write_header()?.write_image_data(pixels)?;
    Ok(())
}
