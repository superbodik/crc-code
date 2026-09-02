use std::sync::Arc;
use std::time::{Duration, Instant};

use crc_editor::{AUTOSAVE_IDLE_MS, Motion};
use crc_theme::{Density, Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::{self, CodeMetrics, Edge, WindowControl};
use crc_ui::{Shell, ShellState, TextRun, WindowRenderer};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, ModifiersState};
use winit::window::{CursorIcon, ResizeDirection, Window, WindowId};

use crate::input::{Command, density_shortcut, resolve};
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
    frames: u32,
    smoke: bool,
}

impl App {
    pub fn new(session: Session, smoke: bool) -> Self {
        Self {
            session,
            theme: Theme::dark(),
            state: ShellState::default(),
            metrics: CodeMetrics::default(),
            modifiers: ModifiersState::empty(),
            cursor: (-1.0, -1.0),
            dragging: false,
            last_title_click: None,
            last_edit: None,
            window: None,
            renderer: None,
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

        let titlebar = self.layout().titlebar;
        let control = view::control_at(titlebar, x, y);
        if control != self.session.view.hovered_control {
            self.session.view.hovered_control = control;
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

        if let Some(sidebar) = layout.sidebar
            && sidebar.contains(x, y)
        {
            let metrics = self.theme.metrics();
            if let Some(row) = view::explorer_row(sidebar, &metrics, y)
                && self.session.open_row(row)
            {
                self.last_edit = None;
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
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(last) = self.last_edit else {
            return;
        };
        let idle = Duration::from_millis(AUTOSAVE_IDLE_MS);
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
            WindowEvent::CloseRequested => {
                let _ = self.session.save();
                event_loop.exit();
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::Focused(focused) => {
                self.session.view.focused = focused;
                if !focused {
                    self.session.view.hovered_control = None;
                    let _ = self.session.save();
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
                self.scroll(delta);
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
                if let Some(command) = self.command_for(&logical_key) {
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
        density_shortcut(key, self.modifiers).or_else(|| resolve(key, self.modifiers, self.rows()))
    }
}
