use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowControl {
    Close,
    Minimize,
    Maximize,
}

impl WindowControl {
    pub const ALL: [WindowControl; 3] = [
        WindowControl::Close,
        WindowControl::Minimize,
        WindowControl::Maximize,
    ];

    const fn order(self) -> usize {
        match self {
            WindowControl::Close => 0,
            WindowControl::Minimize => 1,
            WindowControl::Maximize => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub const CONTROL_DIAMETER: f32 = 11.0;
pub const CONTROL_SPACING: f32 = 18.0;
pub const CONTROL_INSET: f32 = 14.0;
const CONTROL_HIT_PADDING: f32 = 3.0;
pub const RESIZE_MARGIN: f32 = 6.0;

pub fn control_rect(titlebar: Rect, control: WindowControl) -> Rect {
    Rect::new(
        titlebar.x + CONTROL_INSET + control.order() as f32 * CONTROL_SPACING,
        titlebar.y + (titlebar.height - CONTROL_DIAMETER) / 2.0,
        CONTROL_DIAMETER,
        CONTROL_DIAMETER,
    )
}

pub fn control_at(titlebar: Rect, x: f32, y: f32) -> Option<WindowControl> {
    if !titlebar.contains(x, y) {
        return None;
    }
    WindowControl::ALL.into_iter().find(|control| {
        control_rect(titlebar, *control)
            .inset(-CONTROL_HIT_PADDING)
            .contains(x, y)
    })
}

pub fn is_drag_handle(titlebar: Rect, x: f32, y: f32) -> bool {
    titlebar.contains(x, y) && control_at(titlebar, x, y).is_none()
}

pub fn resize_edge(window: Rect, x: f32, y: f32, margin: f32) -> Option<Edge> {
    if !window.contains(x, y) {
        return None;
    }

    let left = x - window.x < margin;
    let right = window.right() - x <= margin;
    let top = y - window.y < margin;
    let bottom = window.bottom() - y <= margin;

    Some(match (top, bottom, left, right) {
        (true, _, true, _) => Edge::TopLeft,
        (true, _, _, true) => Edge::TopRight,
        (_, true, true, _) => Edge::BottomLeft,
        (_, true, _, true) => Edge::BottomRight,
        (true, ..) => Edge::Top,
        (_, true, ..) => Edge::Bottom,
        (_, _, true, _) => Edge::Left,
        (_, _, _, true) => Edge::Right,
        _ => return None,
    })
}
