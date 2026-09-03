use std::sync::Arc;
use std::time::{Duration, Instant};

use crc_config::{Keymap, Settings};
use crc_editor::Motion;
use crc_theme::{Density, Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::{
    self, Action, CodeMetrics, Edge, PaletteView, RecentEntry, TabHit, WelcomeView, WindowControl,
    palette, settings as settings_view, tabs, welcome,
};
use crc_ui::{Shell, ShellState, TextRun, WindowRenderer};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{CursorIcon, ResizeDirection, Window, WindowId};

use crate::input::{Command, command_named, resolve};
use crate::session::Session;

const DOUBLE_CLICK: Duration = Duration::from_millis(400);
const WHEEL_LINES: f32 = 3.0;

pub struct App {
    session: Session,
    theme: Theme,
    state: ShellState,
    metrics: CodeMetrics,
    modifiers: ModifiersState,
    cursor: (f32, f32),
    dragging: bool,
    last_title_click: Option<Instant>,
    last_edit: Option<Instant>,
    window: Option<Arc<Window>>,
    renderer: Option<WindowRenderer>,
    actions: Vec<Action>,
    settings: Settings,
    keymap: Keymap,
    frames: u32,
    smoke: bool,
}

impl App {
    pub fn new(session: Session, smoke: bool) -> Self {
        let (settings, complaint) = Settings::load(&crc_config::settings_file());
        if let Some(complaint) = complaint {
            eprintln!("settings could not be read, using defaults: {complaint}");
        }
        let (keymap, rejected) = settings.keymap();
        for spec in &rejected {
            eprintln!("key binding \"{spec}\" makes no sense and was skipped");
        }

        let theme = Theme::new(match settings.appearance.as_str() {
            "light" => crc_theme::Appearance::Light,
            _ => crc_theme::Appearance::Dark,
        })
        .with_density(match settings.density.as_str() {
            "calm" => Density::Calm,
            "dense" => Density::Dense,
            _ => Density::Balanced,
        });

        let state = ShellState {
            rail: settings.visible.rail,
            sidebar_open: settings.visible.explorer,
            tabs: settings.visible.tabs,
            breadcrumbs: settings.visible.breadcrumbs,
            minimap: settings.visible.minimap,
            panel: settings.visible.panel,
            status_bar: settings.visible.status_bar,
            ..ShellState::default()
        };

        Self {
            session,
            theme,
            state,
            metrics: CodeMetrics::default(),
            modifiers: ModifiersState::empty(),
            cursor: (-1.0, -1.0),
            dragging: false,
            last_title_click: None,
            last_edit: None,
            window: None,
            renderer: None,
            actions: actions(&keymap),
            settings,
            keymap,
            frames: 0,
            smoke,
        }
    }

    fn window_rect(&self) -> Rect {
        let (width, height) = self
            .renderer
            .as_ref()
            .map(|renderer| renderer.size())
            .unwrap_or((1440, 900));
        Rect::from_size(width as f32, height as f32)
    }

    fn layout(&self) -> Shell {
        Shell::compute(self.window_rect(), &self.theme, &self.state)
    }

    fn rows(&self) -> usize {
        self.metrics.rows(self.layout().buffer.height)
    }

    fn calibrate(&mut self) {
        let scale = self
            .window
            .as_ref()
            .map(|window| window.scale_factor() as f32)
            .unwrap_or(1.0);
        self.theme = self.theme.with_scale(scale);

        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let probe = TextRun::new(
            "0000000000",
            Rect::new(0.0, 0.0, 1000.0, 100.0),
            self.theme.type_scale.code,
            Rgba::hex(0x000000),
        )
        .mono();
        let (width, _) = renderer.measure(&probe);
        self.metrics = CodeMetrics {
            char_width: width / 10.0,
            line_height: self.theme.type_scale.code * crc_theme::typography::LINE_HEIGHT_CODE,
        };
    }

    fn redraw(&mut self) {
        let layout = self.layout();
        let frame = view::draw(&layout, &self.theme, &self.session.view, self.metrics);

        if let Some(renderer) = self.renderer.as_mut()
            && let Err(error) = renderer.render(&frame)
        {
            tracing::error!("frame failed: {error}");
        }
        self.frames += 1;
    }

    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn touched(&mut self) {
        self.last_edit = Some(Instant::now());
    }

    fn apply(&mut self, command: Command, event_loop: &ActiveEventLoop) {
        let rows = self.rows();
        match command {
            Command::Quit => event_loop.exit(),
            Command::OpenPalette => self.toggle_palette(),
            Command::OpenFolder => self.pick_folder(),
            Command::ShowWelcome => self.show_welcome(),
            Command::OpenSettings => self.toggle_settings(),
            Command::CloseTab => {
                if let Some(index) = self.session.active_tab() {
                    self.session.close_tab(index);
                    self.last_edit = None;
                    self.refresh_welcome();
                }
            }
            Command::ToggleZen => self.theme.zen = !self.theme.zen,
            Command::ToggleSidebar => self.state.sidebar_open = !self.state.sidebar_open,
            Command::ToggleAppearance => {
                self.theme = self.theme.with_appearance(self.theme.appearance.flipped());
            }
            Command::Density(level) => {
                self.theme.density = match level {
                    1 => Density::Calm,
                    3 => Density::Dense,
                    _ => Density::Balanced,
                };
            }
            Command::Save => match self.session.save() {
                Ok(_) => self.last_edit = None,
                Err(error) => tracing::error!("save failed: {error}"),
            },
            Command::Move { motion, extend } => {
                if let Some(document) = self.session.document() {
                    document.move_cursor(motion, extend);
                    if !matches!(motion, Motion::Left | Motion::Right) {
                        document.commit();
                    }
                }
                self.session.sync();
                self.session.follow_cursor(rows);
            }
            Command::Insert(text) => {
                if let Some(document) = self.session.document() {
                    document.insert(&text);
                }
                self.session.sync();
                self.session.follow_cursor(rows);
                self.touched();
            }
            Command::Backspace => {
                if let Some(document) = self.session.document() {
                    document.backspace();
                }
                self.session.sync();
                self.session.follow_cursor(rows);
                self.touched();
            }
            Command::Delete => {
                if let Some(document) = self.session.document() {
                    document.delete();
                }
                self.session.sync();
                self.session.follow_cursor(rows);
                self.touched();
            }
            Command::Undo => {
                if let Some(document) = self.session.document() {
                    document.undo();
                }
                self.session.sync();
                self.session.follow_cursor(rows);
                self.touched();
            }
            Command::Redo => {
                if let Some(document) = self.session.document() {
                    document.redo();
                }
                self.session.sync();
                self.session.follow_cursor(rows);
                self.touched();
            }
            Command::SelectAll => {
                if let Some(document) = self.session.document() {
                    document.select_all();
                }
                self.session.sync();
            }
        }
    }

    fn resize_margin(&self) -> f32 {
        view::controls::RESIZE_MARGIN * self.theme.scale
    }

    fn place_caret(&mut self, x: f32, y: f32, extend: bool) {
        let layout = self.layout();
        let scroll = self.session.view.scroll_line;
        let point = view::buffer_point(layout.buffer, self.metrics, scroll, x, y);

        if let Some(document) = self.session.document() {
            let offset = document.offset_at(point);
            document.move_cursor(Motion::To(offset), extend);
            if !extend {
                document.commit();
            }
        }
        self.session.sync();
    }

    fn hover(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);

        if self.dragging {
            self.place_caret(x, y, true);
            self.request_redraw();
            return;
        }

        let layout = self.layout();
        if self.session.view.settings.is_some() && !layout.titlebar.contains(x, y) {
            self.settings_hover(x, y);
        } else if self.session.view.welcome.is_some() && !layout.titlebar.contains(x, y) {
            self.welcome_hover(x, y);
        }

        let control = view::control_at(layout.titlebar, x, y);
        if control != self.session.view.hovered_control {
            self.session.view.hovered_control = control;
            self.request_redraw();
        }

        let tab = match tabs::hit(
            layout.tabs,
            &self.session.view.tabs,
            &self.theme.type_scale,
            x,
            y,
        ) {
            Some(TabHit::Select(index)) | Some(TabHit::Close(index)) => Some(index),
            None => None,
        };
        if tab != self.session.view.hovered_tab {
            self.session.view.hovered_tab = tab;
            self.request_redraw();
        }

        let Some(window) = self.window.as_ref() else {
            return;
        };
        let edge = if window.is_maximized() {
            None
        } else {
            view::resize_edge(self.window_rect(), x, y, self.resize_margin())
        };
        window.set_cursor(match edge {
            Some(Edge::Top | Edge::Bottom) => CursorIcon::NsResize,
            Some(Edge::Left | Edge::Right) => CursorIcon::EwResize,
            Some(Edge::TopLeft | Edge::BottomRight) => CursorIcon::NwseResize,
            Some(Edge::TopRight | Edge::BottomLeft) => CursorIcon::NeswResize,
            None if self.layout().buffer.contains(x, y) => CursorIcon::Text,
            None => CursorIcon::Default,
        });
    }

    fn toggle_maximized(&mut self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let maximized = !window.is_maximized();
        window.set_maximized(maximized);
        self.session.view.maximized = maximized;
    }

    fn press(&mut self, event_loop: &ActiveEventLoop) {
        let (x, y) = self.cursor;
        let layout = self.layout();

        if self.session.view.settings.is_some()
            && !layout.titlebar.contains(x, y)
            && self.settings_press(x, y)
        {
            return;
        }

        if self.session.view.welcome.is_some()
            && !layout.titlebar.contains(x, y)
            && self.welcome_press(x, y)
        {
            return;
        }

        if let Some(state) = self.session.view.palette.as_ref() {
            let panel = palette::frame(layout.window, state.rows.len(), self.theme.scale);
            let chosen = palette::row_at(panel, state.rows.len(), self.theme.scale, x, y)
                .and_then(|index| state.rows.get(index).map(|row| row.id));

            self.session.view.palette = None;
            if let Some(id) = chosen
                && let Some(command) = command_named(id)
            {
                self.apply(command, event_loop);
            }
            return;
        }

        let Some(window) = self.window.as_ref() else {
            return;
        };

        if !window.is_maximized()
            && let Some(edge) = view::resize_edge(self.window_rect(), x, y, self.resize_margin())
        {
            let direction = match edge {
                Edge::Top => ResizeDirection::North,
                Edge::Bottom => ResizeDirection::South,
                Edge::Left => ResizeDirection::West,
                Edge::Right => ResizeDirection::East,
                Edge::TopLeft => ResizeDirection::NorthWest,
                Edge::TopRight => ResizeDirection::NorthEast,
                Edge::BottomLeft => ResizeDirection::SouthWest,
                Edge::BottomRight => ResizeDirection::SouthEast,
            };
            let _ = window.drag_resize_window(direction);
            return;
        }

        if let Some(control) = view::control_at(layout.titlebar, x, y) {
            match control {
                WindowControl::Close => event_loop.exit(),
                WindowControl::Minimize => window.set_minimized(true),
                WindowControl::Maximize => self.toggle_maximized(),
            }
            return;
        }

        if view::is_drag_handle(layout.titlebar, x, y) {
            let now = Instant::now();
            let double = self
                .last_title_click
                .is_some_and(|last| now.duration_since(last) < DOUBLE_CLICK);
            self.last_title_click = Some(now);

            if double {
                self.last_title_click = None;
                self.toggle_maximized();
            } else {
                let _ = window.drag_window();
            }
            return;
        }

        match tabs::hit(
            layout.tabs,
            &self.session.view.tabs,
            &self.theme.type_scale,
            x,
            y,
        ) {
            Some(TabHit::Close(index)) => {
                self.session.close_tab(index);
                self.last_edit = None;
                self.refresh_welcome();
                return;
            }
            Some(TabHit::Select(index)) => {
                self.session.activate_tab(index);
                return;
            }
            None if layout.tabs.contains(x, y) => return,
            None => {}
        }

        if let Some(sidebar) = layout.sidebar
            && sidebar.contains(x, y)
        {
            let metrics = self.theme.metrics();
            if let Some(row) = view::explorer_row(sidebar, &metrics, y)
                && self.session.open_row(row)
            {
                self.last_edit = None;
                self.refresh_welcome();
            }
            return;
        }

        if layout.buffer.contains(x, y) || layout.gutter.contains(x, y) {
            self.dragging = true;
            self.place_caret(x, y, self.modifiers.shift_key());
        }
    }

    fn scroll(&mut self, delta: MouseScrollDelta) {
        let lines = match delta {
            MouseScrollDelta::LineDelta(_, y) => -y * WHEEL_LINES,
            MouseScrollDelta::PixelDelta(position) => {
                -(position.y as f32) / self.metrics.line_height.max(1.0)
            }
        };
        self.session.scroll_by(lines.round() as isize);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title(format!("CRC Code — {}", self.session.view.project))
            .with_decorations(false)
            .with_inner_size(winit::dpi::LogicalSize::new(1440.0, 900.0));

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                tracing::error!("no window: {error}");
                event_loop.exit();
                return;
            }
        };

        match WindowRenderer::new(window.clone()) {
            Ok(renderer) => {
                println!("gpu: {}", renderer.adapter());
                let (sans, mono) = renderer.fonts();
                println!("fonts: {sans} / {mono}");
                println!("workspace: {}", self.session.root().display());
                println!("scale: {}", window.scale_factor());
                self.renderer = Some(renderer);
            }
            Err(error) => {
                eprintln!("renderer failed: {error}");
                event_loop.exit();
                return;
            }
        }

        self.window = Some(window);
        self.session.view.focused = true;
        self.calibrate();
        self.refresh_welcome();
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.session.save_all();
        self.store();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(last) = self.last_edit else {
            return;
        };
        let idle = Duration::from_millis(self.settings.autosave_ms);
        let elapsed = last.elapsed();

        if elapsed < idle {
            event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + (idle - elapsed)));
            return;
        }

        self.last_edit = None;
        event_loop.set_control_flow(ControlFlow::Wait);
        match self.session.save() {
            Ok(true) => self.request_redraw(),
            Ok(false) => {}
            Err(error) => tracing::error!("autosave failed: {error}"),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::Focused(focused) => {
                self.session.view.focused = focused;
                if !focused {
                    self.session.view.hovered_control = None;
                    self.session.view.hovered_tab = None;
                    self.session.save_all();
                    self.last_edit = None;
                }
                self.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                self.calibrate();
                self.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.hover(position.x as f32, position.y as f32);
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor = (-1.0, -1.0);
                self.dragging = false;
                if self.session.view.hovered_control.take().is_some() {
                    self.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if self.session.view.settings.is_some() {
                    self.settings_scroll(delta);
                } else {
                    self.scroll(delta);
                }
                self.request_redraw();
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                match state {
                    ElementState::Pressed => self.press(event_loop),
                    ElementState::Released => self.dragging = false,
                }
                self.request_redraw();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
                if let Some(window) = self.window.as_ref() {
                    self.session.view.maximized = window.is_maximized();
                }
                self.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if self.session.view.settings.is_some() {
                    self.settings_key(&logical_key);
                } else if self.session.view.palette.is_some() {
                    self.palette_key(&logical_key, event_loop);
                } else if let Some(command) = self.command_for(&logical_key) {
                    self.apply(command, event_loop);
                }
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
                if !self.smoke {
                    return;
                }
                if self.frames >= 3 {
                    println!("smoke: {} frames drawn", self.frames);
                    event_loop.exit();
                } else {
                    self.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn command_for(&self, key: &Key) -> Option<Command> {
        resolve(key, self.modifiers, self.rows(), &self.keymap)
    }

    fn store(&mut self) {
        self.settings.appearance = match self.theme.appearance {
            crc_theme::Appearance::Light => "light".to_string(),
            crc_theme::Appearance::Dark => "dark".to_string(),
        };
        self.settings.density = match self.theme.density {
            Density::Calm => "calm".to_string(),
            Density::Dense => "dense".to_string(),
            Density::Balanced => "balanced".to_string(),
        };
        self.settings.visible.explorer = self.state.sidebar_open;
        self.settings.visible.rail = self.state.rail;
        self.settings.visible.tabs = self.state.tabs;
        self.settings.visible.breadcrumbs = self.state.breadcrumbs;
        self.settings.visible.minimap = self.state.minimap;
        self.settings.visible.panel = self.state.panel;
        self.settings.visible.status_bar = self.state.status_bar;
        self.settings
            .remember(self.session.root(), crc_config::recent::now());

        if let Err(error) = self.settings.save(&crc_config::settings_file()) {
            tracing::error!("settings not saved: {error}");
        }
    }
}

impl App {
    fn toggle_palette(&mut self) {
        self.session.view.palette = match self.session.view.palette {
            Some(_) => None,
            None => Some(PaletteView {
                query: String::new(),
                rows: palette::filter(&self.actions, ""),
                selected: 0,
            }),
        };
    }

    fn refilter(&mut self) {
        let Some(state) = self.session.view.palette.as_mut() else {
            return;
        };
        state.rows = palette::filter(&self.actions, &state.query);
        state.selected = 0;
    }

    fn palette_key(&mut self, key: &Key, event_loop: &ActiveEventLoop) {
        let control = self.modifiers.control_key();

        match key {
            Key::Named(NamedKey::Escape) => self.session.view.palette = None,
            Key::Named(NamedKey::ArrowDown) => {
                if let Some(state) = self.session.view.palette.as_mut() {
                    state.move_selection(1);
                }
            }
            Key::Named(NamedKey::ArrowUp) => {
                if let Some(state) = self.session.view.palette.as_mut() {
                    state.move_selection(-1);
                }
            }
            Key::Named(NamedKey::Enter) => {
                let chosen = self
                    .session
                    .view
                    .palette
                    .as_ref()
                    .and_then(|state| state.selected_id());
                self.session.view.palette = None;
                if let Some(id) = chosen
                    && let Some(command) = command_named(id)
                {
                    self.apply(command, event_loop);
                }
            }
            Key::Named(NamedKey::Backspace) => {
                if let Some(state) = self.session.view.palette.as_mut() {
                    state.query.pop();
                }
                self.refilter();
            }
            Key::Named(NamedKey::Space) if !control => {
                if let Some(state) = self.session.view.palette.as_mut() {
                    state.query.push(' ');
                }
                self.refilter();
            }
            Key::Character(text) if control => {
                if matches!(text.to_lowercase().as_str(), "k" | "л") {
                    self.session.view.palette = None;
                }
            }
            Key::Character(text) => {
                if let Some(state) = self.session.view.palette.as_mut() {
                    state.query.push_str(text);
                }
                self.refilter();
            }
            _ => {}
        }
    }
}

impl App {
    fn show_welcome(&mut self) {
        let now = crc_config::recent::now();
        let recent = self
            .settings
            .recent
            .iter()
            .take(welcome::MAX_RECENT)
            .map(|entry| RecentEntry {
                name: entry.name.clone(),
                path: entry.path.to_string_lossy().into_owned(),
                when: crc_config::recent::since(entry.opened_at, now),
            })
            .collect();

        let hint = |command: &str| self.keymap.hint(command).unwrap_or_default();
        self.session.view.welcome = Some(WelcomeView {
            recent,
            hints: vec![
                (
                    hint("palette"),
                    "Командная палитра — всё с клавиатуры".to_string(),
                ),
                (hint("open-folder"), "Открыть другой проект".to_string()),
                (hint("zen"), "Zen — панели уходят, остаётся код".to_string()),
                (hint("theme"), "Светлая и тёмная тема".to_string()),
            ],
            hovered: None,
        });
    }

    fn refresh_welcome(&mut self) {
        let empty = self.session.view.tabs.is_empty();
        if empty && self.session.view.welcome.is_none() {
            self.show_welcome();
        } else if !empty {
            self.session.view.welcome = None;
        }
    }

    fn pick_folder(&mut self) {
        let picked = rfd::FileDialog::new()
            .set_title("Открыть папку проекта")
            .set_directory(self.session.root())
            .pick_folder();

        if let Some(path) = picked {
            self.open_project(&path);
        }
    }

    fn open_project(&mut self, path: &std::path::Path) {
        self.session.save_all();
        self.settings
            .remember(self.session.root(), crc_config::recent::now());

        match Session::open(path) {
            Ok(session) => {
                self.session = session;
                self.settings.remember(path, crc_config::recent::now());
                self.last_edit = None;
                self.refresh_welcome();
            }
            Err(error) => {
                tracing::error!("could not open {}: {error}", path.display());
            }
        }
    }

    fn welcome_press(&mut self, x: f32, y: f32) -> bool {
        let Some(state) = self.session.view.welcome.as_ref() else {
            return false;
        };
        let layout = self.layout();
        let window = Rect::new(
            layout.window.x,
            layout.titlebar.bottom(),
            layout.window.width,
            layout.window.bottom() - layout.titlebar.bottom(),
        );
        let placed = welcome::layout(window, state, self.theme.scale);

        match welcome::target_at(&placed, x, y) {
            Some(welcome::Target::OpenFolder) => {
                self.pick_folder();
                true
            }
            Some(welcome::Target::Recent(index)) => {
                if let Some(entry) = self.settings.recent.get(index) {
                    let path = entry.path.clone();
                    self.open_project(&path);
                }
                true
            }
            None => true,
        }
    }

    fn welcome_hover(&mut self, x: f32, y: f32) -> bool {
        let Some(state) = self.session.view.welcome.as_ref() else {
            return false;
        };
        let layout = self.layout();
        let window = Rect::new(
            layout.window.x,
            layout.titlebar.bottom(),
            layout.window.width,
            layout.window.bottom() - layout.titlebar.bottom(),
        );
        let placed = welcome::layout(window, state, self.theme.scale);
        let target = welcome::target_at(&placed, x, y);

        if let Some(state) = self.session.view.welcome.as_mut()
            && state.hovered != target
        {
            state.hovered = target;
            self.request_redraw();
        }
        true
    }
}


impl App {
    fn toggle_settings(&mut self) {
        if self.session.view.settings.is_some() {
            self.session.view.settings = None;
            return;
        }
        self.session.view.settings = Some(settings_view::SettingsView {
            section: settings_view::Section::Appearance,
            query: String::new(),
            toggles: self.toggles(),
            bindings: self.binding_rows(),
            capturing: None,
            hovered: None,
            scroll: 0,
        });
    }

    fn toggles(&self) -> Vec<settings_view::Toggle> {
        use settings_view::Toggle;
        vec![
            Toggle::new(
                "dark",
                "Тёмная тема",
                "Светлый и тёмный набор цветов",
                matches!(self.theme.appearance, crc_theme::Appearance::Dark),
            ),
            Toggle::new(
                "rail",
                "Рейка действий",
                "Узкая полоса у левого края",
                self.state.rail,
            ),
            Toggle::new(
                "explorer",
                "Проводник",
                "Дерево файлов проекта",
                self.state.sidebar_open,
            ),
            Toggle::new(
                "tabs",
                "Вкладки",
                "Строка с открытыми файлами",
                self.state.tabs,
            ),
            Toggle::new(
                "breadcrumbs",
                "Путь над файлом",
                "Проект и имя файла",
                self.state.breadcrumbs,
            ),
            Toggle::new(
                "minimap",
                "Мини-карта",
                "Полоса обзора справа от кода",
                self.state.minimap,
            ),
            Toggle::new(
                "panel",
                "Нижняя панель",
                "Терминал, проблемы, вывод",
                self.state.panel,
            ),
            Toggle::new(
                "status",
                "Статус-бар",
                "Строка внизу окна",
                self.state.status_bar,
            ),
        ]
    }

    fn binding_rows(&self) -> Vec<settings_view::BindingRow> {
        let mut rows: Vec<settings_view::BindingRow> = self
            .actions
            .iter()
            .map(|action| settings_view::BindingRow {
                command: action.id.to_string(),
                title: action.title.clone(),
                keys: self.keymap.hint(action.id).unwrap_or_default(),
                clash: None,
                changed: self
                    .settings
                    .keys
                    .iter()
                    .any(|binding| binding.command == action.id),
            })
            .collect();
        settings_view::mark_clashes(&mut rows);
        rows
    }

    fn flip_toggle(&mut self, index: usize) {
        let Some(id) = self
            .session
            .view
            .settings
            .as_ref()
            .and_then(|state| state.toggles.get(index))
            .map(|toggle| toggle.id.clone())
        else {
            return;
        };

        match id.as_str() {
            "dark" => self.theme = self.theme.with_appearance(self.theme.appearance.flipped()),
            "rail" => self.state.rail = !self.state.rail,
            "explorer" => self.state.sidebar_open = !self.state.sidebar_open,
            "tabs" => self.state.tabs = !self.state.tabs,
            "breadcrumbs" => self.state.breadcrumbs = !self.state.breadcrumbs,
            "minimap" => self.state.minimap = !self.state.minimap,
            "panel" => self.state.panel = !self.state.panel,
            "status" => self.state.status_bar = !self.state.status_bar,
            _ => return,
        }

        let refreshed = self.toggles();
        if let Some(state) = self.session.view.settings.as_mut() {
            state.toggles = refreshed;
        }
        self.store();
    }

    fn settings_key(&mut self, key: &Key) {
        let capturing = self
            .session
            .view
            .settings
            .as_ref()
            .and_then(|state| state.capturing);

        let Some(index) = capturing else {
            self.browse_keys(key);
            return;
        };

        if matches!(key, Key::Named(NamedKey::Escape)) {
            if let Some(state) = self.session.view.settings.as_mut() {
                state.capturing = None;
            }
            return;
        }

        let Some(chord) = crate::input::chord(key, self.modifiers) else {
            return;
        };

        if let Some(state) = self.session.view.settings.as_mut() {
            if let Some(row) = state.bindings.get_mut(index) {
                row.keys = chord.label();
                row.changed = true;
            }
            state.capturing = None;
            settings_view::mark_clashes(&mut state.bindings);
        }
        self.rebind();
    }

    fn rebind(&mut self) {
        let Some(state) = self.session.view.settings.as_ref() else {
            return;
        };
        let taken: Vec<crc_config::Chord> = state
            .bindings
            .iter()
            .filter_map(|row| crc_config::Chord::parse(&row.keys))
            .collect();

        let mut keys: Vec<crc_config::Binding> = crc_config::keymap::defaults()
            .into_iter()
            .filter(|binding| {
                crc_config::Chord::parse(&binding.keys)
                    .is_some_and(|chord| !taken.contains(&chord))
            })
            .map(|binding| crc_config::Binding::new(binding.keys, ""))
            .collect();

        keys.extend(
            state
                .bindings
                .iter()
                .filter(|row| !row.keys.is_empty())
                .map(|row| crc_config::Binding::new(row.keys.clone(), row.command.clone())),
        );
        self.settings.keys = keys;

        let (keymap, rejected) = self.settings.keymap();
        for spec in &rejected {
            tracing::warn!("key binding {spec} makes no sense and was skipped");
        }
        self.keymap = keymap;
        self.actions = actions(&self.keymap);
        self.store();
    }

    fn settings_press(&mut self, x: f32, y: f32) -> bool {
        let Some(state) = self.session.view.settings.as_ref() else {
            return false;
        };
        let placed = settings_view::layout(self.layout().window, state, self.theme.scale);

        match settings_view::target_at(&placed, state, x, y) {
            Some(settings_view::Target::Close) | None => self.session.view.settings = None,
            Some(settings_view::Target::Section(index)) => {
                if let Some(section) = settings_view::Section::ALL.get(index).copied()
                    && let Some(state) = self.session.view.settings.as_mut()
                {
                    state.section = section;
                    state.scroll = 0;
                    state.capturing = None;
                    state.query.clear();
                }
            }
            Some(settings_view::Target::Toggle(index)) => self.flip_toggle(index),
            Some(settings_view::Target::Binding(index)) => {
                if let Some(state) = self.session.view.settings.as_mut() {
                    state.capturing = Some(index);
                }
            }
            Some(settings_view::Target::Reset) => self.reset_keys(),
            Some(settings_view::Target::Search) => {
                if let Some(state) = self.session.view.settings.as_mut() {
                    state.capturing = None;
                }
            }
        }
        true
    }

    fn settings_hover(&mut self, x: f32, y: f32) {
        let Some(state) = self.session.view.settings.as_ref() else {
            return;
        };
        let placed = settings_view::layout(self.layout().window, state, self.theme.scale);
        let target = settings_view::target_at(&placed, state, x, y);

        if let Some(state) = self.session.view.settings.as_mut()
            && state.hovered != target
        {
            state.hovered = target;
            self.request_redraw();
        }
    }

    fn settings_scroll(&mut self, delta: MouseScrollDelta) {
        let lines = match delta {
            MouseScrollDelta::LineDelta(_, y) => -y * WHEEL_LINES,
            MouseScrollDelta::PixelDelta(position) => -(position.y as f32) / 40.0,
        };
        let lines = lines.round() as isize;

        let Some(state) = self.session.view.settings.as_ref() else {
            return;
        };
        let placed = settings_view::layout(self.layout().window, state, self.theme.scale);
        let last = state
            .rows()
            .saturating_sub(settings_view::visible_rows(&placed));

        if let Some(state) = self.session.view.settings.as_mut() {
            let next = state.scroll as isize + lines;
            state.scroll = next.clamp(0, last as isize) as usize;
        }
    }
}


impl App {
    fn browse_keys(&mut self, key: &Key) {
        if matches!(key, Key::Named(NamedKey::Escape)) {
            self.session.view.settings = None;
            return;
        }

        if let Some(chord) = crate::input::chord(key, self.modifiers)
            && self.keymap.command(&chord) == Some("settings")
        {
            self.session.view.settings = None;
            return;
        }

        let Some(state) = self.session.view.settings.as_mut() else {
            return;
        };
        if state.section != settings_view::Section::Keys {
            return;
        }

        let control = self.modifiers.control_key() || self.modifiers.alt_key();
        match key {
            Key::Named(NamedKey::Backspace) => {
                state.query.pop();
            }
            Key::Named(NamedKey::Space) if !control => state.query.push(' '),
            Key::Character(text) if !control => state.query.push_str(text),
            _ => return,
        }
        state.scroll = 0;
    }

    fn reset_keys(&mut self) {
        self.settings.keys.clear();

        let (keymap, _) = self.settings.keymap();
        self.keymap = keymap;
        self.actions = actions(&self.keymap);

        let rows = self.binding_rows();
        if let Some(state) = self.session.view.settings.as_mut() {
            state.bindings = rows;
            state.capturing = None;
            state.scroll = 0;
        }
        self.store();
    }
}

fn actions(keymap: &Keymap) -> Vec<Action> {
    let hint = |id: &str| keymap.hint(id).unwrap_or_default();
    vec![
        Action::new("open-folder", "Открыть папку проекта", "Файл").hint(hint("open-folder")),
        Action::new("settings", "Настройки", "Вид").hint(hint("settings")),
        Action::new("welcome", "Показать стартовый экран", "Файл").hint(hint("welcome")),
        Action::new("save", "Сохранить файл", "Файл").hint(hint("save")),
        Action::new("close-tab", "Закрыть вкладку", "Файл").hint(hint("close-tab")),
        Action::new("undo", "Отменить правку", "Правка").hint(hint("undo")),
        Action::new("redo", "Вернуть правку", "Правка").hint(hint("redo")),
        Action::new("select-all", "Выделить всё", "Правка").hint(hint("select-all")),
        Action::new("theme", "Переключить светлую и тёмную тему", "Вид").hint(hint("theme")),
        Action::new("sidebar", "Показать или скрыть проводник", "Вид").hint(hint("sidebar")),
        Action::new("zen", "Zen — оставить только код", "Вид").hint(hint("zen")),
        Action::new("calm", "Плотность: спокойно", "Вид").hint(hint("calm")),
        Action::new("balanced", "Плотность: сбалансированно", "Вид").hint(hint("balanced")),
        Action::new("dense", "Плотность: максимум мощи", "Вид").hint(hint("dense")),
        Action::new("quit", "Выйти из редактора", "Файл").hint(hint("quit")),
    ]
}
