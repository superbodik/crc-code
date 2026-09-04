pub mod agent;
pub mod controls;
pub mod explorer;
pub mod find;
pub mod hit;
pub mod logo;
pub mod menu;
pub mod palette;
pub mod panel;
pub mod prompt;
pub mod rail;
pub mod search;
pub mod selection;
pub mod settings;
pub mod shell;
pub mod state;
pub mod tabs;
pub mod welcome;

pub use controls::{Edge, WindowControl, control_at, control_rect, is_drag_handle, resize_edge};
pub use explorer::tree;
pub use hit::{
    ExplorerButton, buffer_point, explorer_button, explorer_button_at, explorer_header_height,
    explorer_row,
};
pub use rail::RailAction;
pub use search::{SearchRow, SearchView};
pub use logo::{Mark, clear_space, mark};
pub use find::FindView;
pub use agent::AgentView;
pub use menu::{MenuAction, MenuItem, MenuView};
pub use palette::{Action, PaletteView};
pub use prompt::{PromptKind, PromptView};
pub use panel::{PanelTab, PanelView, Problem};
pub use selection::{Band, bands};
pub use settings::{BindingRow, Section, SettingsView, Toggle};
pub use shell::draw;
pub use state::{CodeMetrics, EditorView, FileEntry, Tab, VisibleText};
pub use tabs::TabHit;
pub use welcome::{RecentEntry, Target, WelcomeView};
