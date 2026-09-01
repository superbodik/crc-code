use std::sync::Arc;

use crc_theme::{Density, Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::{self, CodeMetrics};
use crc_ui::{Shell, ShellState, TextRun, WindowRenderer};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use crate::session::Session;

pub struct App {
    session: Session,
    theme: Theme,
    state: ShellState,
    metrics: CodeMetrics,
    modifiers: ModifiersState,
    window: Option<Arc<Window>>,
    renderer: Option<WindowRenderer>,
    frames: u32,
    smoke: bool,
}

impl App {
    pub fn new(session: Session, smoke: bool) -> Self {
        Self {
            session,
            theme: Theme::light(),
            state: ShellState::default(),
            metrics: CodeMetrics::default(),
            modifiers: ModifiersState::empty(),
            window: None,
            renderer: None,
            frames: 0,
            smoke,
        }
    }

    fn layout(&self) -> Shell {
        let (width, height) = self
            .renderer
            .as_ref()
            .map(|renderer| renderer.size())
            .unwrap_or((1440, 900));
        Shell::compute(
            Rect::from_size(width as f32, height as f32),
            &self.theme,
            &self.state,
        )
    }

    fn calibrate(&mut self) {
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

    fn rows(&self) -> usize {
        self.metrics.rows(self.layout().buffer.height)
    }

    fn key(&mut self, event_loop: &ActiveEventLoop, key: &Key) {
        let control = self.modifiers.control_key();
        let alt = self.modifiers.alt_key();

        match key {
            Key::Named(NamedKey::Escape) => event_loop.exit(),
            Key::Named(NamedKey::ArrowDown) => self.session.move_cursor(1, self.rows()),
            Key::Named(NamedKey::ArrowUp) => self.session.move_cursor(-1, self.rows()),
            Key::Named(NamedKey::PageDown) => {
                let rows = self.rows() as isize;
                self.session.move_cursor(rows, self.rows());
            }
            Key::Named(NamedKey::PageUp) => {
                let rows = self.rows() as isize;
                self.session.move_cursor(-rows, self.rows());
            }
            Key::Character(character) => match character.as_str() {
                "z" | "Z" | "я" | "Я" if alt => self.theme.zen = !self.theme.zen,
                "b" | "B" | "и" | "И" if control => {
                    self.state.sidebar_open = !self.state.sidebar_open
                }
                "1" => self.theme.density = Density::Calm,
                "2" => self.theme.density = Density::Balanced,
                "3" => self.theme.density = Density::Dense,
                "q" | "Q" if control => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title(format!("CRC Code — {}", self.session.view.project))
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
                self.renderer = Some(renderer);
            }
            Err(error) => {
                eprintln!("renderer failed: {error}");
                event_loop.exit();
                return;
            }
        }

        self.window = Some(window);
        self.calibrate();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
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
                self.key(event_loop, &logical_key);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
                if !self.smoke {
                    return;
                }
                if self.frames >= 3 {
                    println!("smoke: {} frames drawn", self.frames);
                    event_loop.exit();
                } else if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
