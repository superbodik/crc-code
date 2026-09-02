use crc_theme::{CONTROL_RING, Rgba, Theme, Weight};

use crate::geometry::Rect;
use crate::gpu::{Frame, Quad, Span, TextAlign, TextRun};
use crate::layout::Shell;
use crate::view::controls::{WindowControl, control_rect};
use crate::view::state::{CodeMetrics, EditorView};

pub fn draw(layout: &Shell, theme: &Theme, view: &EditorView, metrics: CodeMetrics) -> Frame {
    let mut frame = Frame::new(theme.chrome.backdrop);

    titlebar(&mut frame, layout, theme, view);
    rail(&mut frame, layout, theme);
    sidebar(&mut frame, layout, theme, view);
    tabs(&mut frame, layout, theme, view);
    breadcrumbs(&mut frame, layout, theme, view);
    editor(&mut frame, layout, theme, view, metrics);
    minimap(&mut frame, layout, theme, view);
    panel(&mut frame, layout, theme);
    aside(&mut frame, layout, theme);
    statusbar(&mut frame, layout, theme, view);

    frame
}

fn hairline_bottom(frame: &mut Frame, rect: Rect, colour: Rgba) {
    frame.quad(Quad::filled(
        Rect::new(rect.x, rect.bottom() - 1.0, rect.width, 1.0),
        colour,
    ));
}

fn hairline_right(frame: &mut Frame, rect: Rect, colour: Rgba) {
    frame.quad(Quad::filled(
        Rect::new(rect.right() - 1.0, rect.y, 1.0, rect.height),
        colour,
    ));
}

fn titlebar(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let bar = layout.titlebar;
    let scale = theme.type_scale;

    frame.quad(Quad::filled(bar, theme.chrome.panel));
    hairline_bottom(frame, bar, theme.chrome.border);

    for control in WindowControl::ALL {
        let rect = control_rect(bar, control);
        let fill = if view.focused {
            match control {
                WindowControl::Close => theme.chrome.control_close,
                WindowControl::Minimize => theme.chrome.control_minimize,
                WindowControl::Maximize => theme.chrome.control_maximize,
            }
        } else {
            theme.chrome.control_idle
        };
        let fill = if view.hovered_control == Some(control) {
            fill.shade(0.12)
        } else {
            fill
        };

        frame.quad(
            Quad::filled(rect, fill)
                .rounded(rect.width / 2.0)
                .bordered(1.0, fill.shade(CONTROL_RING)),
        );
    }

    let label = if view.branch.is_empty() {
        view.project.clone()
    } else {
        format!("{}  ·  {}", view.project, view.branch)
    };
    frame.text(
        TextRun::new(
            label,
            bar.inset_by(12.0, 0.0),
            scale.small,
            theme.chrome.text_muted,
        )
        .align(TextAlign::Center)
        .line_height(bar.height),
    );
}

fn rail(frame: &mut Frame, layout: &Shell, theme: &Theme) {
    let Some(rail) = layout.rail else { return };
    frame.quad(Quad::filled(rail, theme.chrome.panel));
    hairline_right(frame, rail, theme.chrome.border);

    let metrics = theme.metrics();
    for index in 0..4 {
        let y = rail.y + metrics.panel_padding + index as f32 * (metrics.row_height + 8.0);
        let colour = if index == 0 {
            theme.chrome.accent
        } else {
            theme.chrome.text_faint
        };
        frame.quad(
            Quad::filled(Rect::new(rail.x + 14.0, y, 16.0, 16.0), colour)
                .rounded(metrics.corner_radius_small * 0.6),
        );
    }
}

fn sidebar(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(sidebar) = layout.sidebar else {
        return;
    };
    let metrics = theme.metrics();
    let scale = theme.type_scale;

    frame.quad(Quad::filled(sidebar, theme.chrome.panel));
    hairline_right(frame, sidebar, theme.chrome.border);

    let header = Rect::new(
        sidebar.x,
        sidebar.y,
        sidebar.width,
        metrics.row_height + 8.0,
    );
    frame.text(
        TextRun::new(
            "Проводник",
            header.inset_by(metrics.panel_padding, 0.0),
            scale.small,
            theme.chrome.text_muted,
        )
        .weight(Weight::Semibold)
        .line_height(header.height),
    );

    let mut y = header.bottom();
    for entry in &view.files {
        if y + metrics.row_height > sidebar.bottom() {
            break;
        }
        let row = Rect::new(sidebar.x, y, sidebar.width, metrics.row_height);

        if entry.selected {
            frame.quad(
                Quad::filled(row.inset_by(4.0, 1.0), theme.chrome.selected)
                    .rounded(metrics.corner_radius_small),
            );
        }

        let indent = metrics.panel_padding + entry.depth as f32 * 12.0;
        let marker = Rect::new(row.x + indent, row.y + row.height / 2.0 - 3.0, 6.0, 6.0);
        frame.quad(
            Quad::filled(
                marker,
                if entry.is_dir {
                    theme.chrome.text_faint
                } else if entry.modified {
                    theme.chrome.warning
                } else {
                    theme.chrome.border
                },
            )
            .rounded(if entry.is_dir { 1.0 } else { 3.0 }),
        );

        let text = Rect::new(
            row.x + indent + 14.0,
            row.y,
            row.width - indent - 20.0,
            row.height,
        );
        frame.text(
            TextRun::new(
                entry.name.clone(),
                text,
                scale.body,
                if entry.selected {
                    theme.chrome.text_strong
                } else {
                    theme.chrome.text
                },
            )
            .line_height(row.height),
        );

        y += metrics.row_height;
    }
}

fn tabs(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let bar = layout.tabs;
    let metrics = theme.metrics();
    let scale = theme.type_scale;

    frame.quad(Quad::filled(bar, theme.chrome.panel));
    hairline_bottom(frame, bar, theme.chrome.border);

    let mut x = bar.x;
    for tab in &view.tabs {
        let width = 24.0 + tab.name.chars().count() as f32 * scale.body * 0.62;
        if x + width > bar.right() {
            break;
        }
        let rect = Rect::new(x, bar.y, width, bar.height);

        if tab.active {
            frame.quad(Quad::filled(rect, theme.chrome.surface));
            frame.quad(Quad::filled(
                Rect::new(rect.x, rect.y, rect.width, 2.0),
                theme.chrome.accent,
            ));
        }

        frame.text(
            TextRun::new(
                tab.name.clone(),
                rect.inset_by(12.0, 0.0),
                scale.small,
                if tab.active {
                    theme.chrome.text_strong
                } else {
                    theme.chrome.text_muted
                },
            )
            .line_height(rect.height),
        );

        if tab.modified {
            frame.quad(
                Quad::filled(
                    Rect::new(
                        rect.right() - 12.0,
                        rect.y + rect.height / 2.0 - 3.0,
                        6.0,
                        6.0,
                    ),
                    theme.chrome.warning,
                )
                .rounded(3.0),
            );
        }

        x += width;
        frame.quad(Quad::filled(
            Rect::new(x - 1.0, bar.y + 6.0, 1.0, bar.height - 12.0),
            theme.chrome.border,
        ));
    }

    let _ = metrics;
}

fn breadcrumbs(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(crumbs) = layout.breadcrumbs else {
        return;
    };
    frame.quad(Quad::filled(crumbs, theme.chrome.surface));

    let trail = view
        .tabs
        .iter()
        .find(|tab| tab.active)
        .map(|tab| format!("{}  ›  {}", view.project, tab.name))
        .unwrap_or_else(|| view.project.clone());

    frame.text(
        TextRun::new(
            trail,
            crumbs.inset_by(theme.metrics().panel_padding, 0.0),
            theme.type_scale.small,
            theme.chrome.text_muted,
        )
        .line_height(crumbs.height),
    );
}

fn editor(
    frame: &mut Frame,
    layout: &Shell,
    theme: &Theme,
    view: &EditorView,
    metrics: CodeMetrics,
) {
    let gutter = layout.gutter;
    let buffer = layout.buffer;
    let scale = theme.type_scale;

    frame.quad(Quad::filled(gutter, theme.chrome.surface));
    frame.quad(Quad::filled(buffer, theme.chrome.surface));

    let rows = metrics.rows(buffer.height);
    let visible = view.visible(rows);

    let cursor_row = view.cursor_line.saturating_sub(visible.first_line);
    if cursor_row < rows {
        let y = buffer.y + cursor_row as f32 * metrics.line_height;
        frame.quad(Quad::filled(
            Rect::new(
                gutter.x,
                y,
                gutter.width + buffer.width,
                metrics.line_height,
            ),
            theme.syntax.current_line,
        ));
    }

    let numbers: String = (0..rows)
        .map(|row| (visible.first_line + row + 1).to_string())
        .take(view.line_count().saturating_sub(visible.first_line))
        .collect::<Vec<_>>()
        .join("\n");

    frame.text(
        TextRun::new(
            numbers,
            Rect::new(gutter.x, buffer.y, gutter.width - 10.0, gutter.height),
            scale.code,
            theme.syntax.line_number,
        )
        .mono()
        .align(TextAlign::End)
        .line_height(metrics.line_height),
    );

    let spans = visible
        .spans
        .iter()
        .map(|(range, highlight)| Span::new(range.clone(), theme.syntax.color(*highlight)))
        .collect();

    frame.text(
        TextRun::new(visible.text, buffer, scale.code, theme.syntax.text)
            .mono()
            .line_height(metrics.line_height)
            .spans(spans),
    );

    if cursor_row < rows {
        let x = buffer.x + view.cursor_column as f32 * metrics.char_width;
        let y = buffer.y + cursor_row as f32 * metrics.line_height;
        frame.quad(Quad::filled(
            Rect::new(x, y + 2.0, 1.5, metrics.line_height - 4.0),
            theme.syntax.caret,
        ));
    }
}

fn minimap(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(minimap) = layout.minimap else {
        return;
    };
    frame.quad(Quad::filled(minimap, theme.chrome.surface));

    let mut y = minimap.y + 4.0;
    for line in view.text.lines() {
        if y > minimap.bottom() - 2.0 {
            break;
        }
        let indent = line.len() - line.trim_start().len();
        let width = (line.trim().len() as f32 * 0.9).min(minimap.width - 12.0);
        if width > 0.0 {
            frame.quad(Quad::filled(
                Rect::new(minimap.x + 6.0 + indent as f32 * 0.9, y, width, 1.5),
                theme.chrome.text_faint,
            ));
        }
        y += 2.5;
    }
}

fn panel(frame: &mut Frame, layout: &Shell, theme: &Theme) {
    let Some(panel) = layout.panel else { return };
    let metrics = theme.metrics();
    let scale = theme.type_scale;

    frame.quad(Quad::filled(panel, theme.chrome.panel));
    frame.quad(Quad::filled(
        Rect::new(panel.x, panel.y, panel.width, 1.0),
        theme.chrome.border,
    ));

    let header = Rect::new(panel.x, panel.y, panel.width, metrics.row_height + 6.0);
    let mut x = header.x + metrics.panel_padding;
    for (index, name) in ["Терминал", "Проблемы", "Вывод", "Тесты"]
        .iter()
        .enumerate()
    {
        let width = name.chars().count() as f32 * scale.small * 0.62 + 18.0;
        let tab = Rect::new(x, header.y + 4.0, width, header.height - 8.0);
        if index == 0 {
            frame.quad(
                Quad::filled(tab, theme.chrome.selected).rounded(metrics.corner_radius_small),
            );
        }
        frame.text(
            TextRun::new(
                *name,
                tab.inset_by(9.0, 0.0),
                scale.small,
                if index == 0 {
                    theme.chrome.text_strong
                } else {
                    theme.chrome.text_muted
                },
            )
            .line_height(tab.height),
        );
        x += width + 4.0;
    }

    let body = Rect::new(
        panel.x + metrics.panel_padding,
        header.bottom(),
        panel.width - metrics.panel_padding * 2.0,
        panel.height - header.height - metrics.panel_padding,
    );
    frame.text(
        TextRun::new(
            "crc ~ $ cargo test\n   Compiling crc-ui v0.1.0\n    Finished in 2.30s",
            body,
            scale.small,
            theme.chrome.text_muted,
        )
        .mono()
        .line_height(scale.small * 1.6),
    );
}

fn aside(frame: &mut Frame, layout: &Shell, theme: &Theme) {
    let Some(aside) = layout.aside else { return };
    frame.quad(Quad::filled(aside, theme.chrome.panel));
    frame.quad(Quad::filled(
        Rect::new(aside.x, aside.y, 1.0, aside.height),
        theme.chrome.border,
    ));
}

fn statusbar(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(bar) = layout.statusbar else { return };
    let metrics = theme.metrics();
    let scale = theme.type_scale;

    frame.quad(Quad::filled(bar, theme.chrome.panel));
    frame.quad(Quad::filled(
        Rect::new(bar.x, bar.y, bar.width, 1.0),
        theme.chrome.border,
    ));

    let left = if view.problems > 0 {
        format!("{}   {} проблемы", view.branch, view.problems)
    } else {
        view.branch.clone()
    };
    frame.text(
        TextRun::new(
            left,
            bar.inset_by(metrics.panel_padding, 0.0),
            scale.small,
            theme.chrome.text_muted,
        )
        .line_height(bar.height),
    );

    frame.text(
        TextRun::new(
            format!(
                "Стр {}, Кол {}   {}   UTF-8   LF",
                view.cursor_line + 1,
                view.cursor_column + 1,
                view.language
            ),
            bar.inset_by(metrics.panel_padding, 0.0),
            scale.small,
            theme.chrome.text_muted,
        )
        .align(TextAlign::End)
        .line_height(bar.height),
    );
}
