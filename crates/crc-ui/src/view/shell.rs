use crc_theme::{Brand, CONTROL_RING, Rgba, Theme, Weight};

use crate::geometry::Rect;
use crate::gpu::{Frame, Quad, Span, TextAlign, TextRun};
use crate::layout::Shell;
use crate::view::controls::{WindowControl, control_rect};
use crate::view::logo;
use crate::view::palette;
use crate::view::selection::bands;
use crate::view::state::{CodeMetrics, EditorView};
use crate::view::tabs;
use crate::icon;
use crate::view::find as find_view;
use crate::view::hit;
use crate::view::menu as menu_view;
use crate::view::panel as panel_view;
use crate::view::prompt as prompt_view;
use crate::view::rail as rail_view;
use crate::view::search as search_view;
use crate::view::settings as settings_view;
use crate::view::welcome;

pub fn draw(layout: &Shell, theme: &Theme, view: &EditorView, metrics: CodeMetrics) -> Frame {
    let mut frame = Frame::new(theme.chrome.backdrop);

    titlebar(&mut frame, layout, theme, view);

    if view.welcome.is_some() {
        welcome_screen(&mut frame, layout, theme, view);
        settings_panel(&mut frame, layout, theme, view);
        command_palette(&mut frame, layout, theme, view);
        context_menu(&mut frame, layout, theme, view);
        name_prompt(&mut frame, layout, theme, view);
        return frame;
    }

    rail(&mut frame, layout, theme, view);
    sidebar(&mut frame, layout, theme, view);
    tabs(&mut frame, layout, theme, view);
    breadcrumbs(&mut frame, layout, theme, view);
    editor(&mut frame, layout, theme, view, metrics);
    minimap(&mut frame, layout, theme, view);
    panel(&mut frame, layout, theme, view, metrics);
    aside(&mut frame, layout, theme);
    statusbar(&mut frame, layout, theme, view);
    find_bar(&mut frame, layout, theme, view);
    settings_panel(&mut frame, layout, theme, view);
    command_palette(&mut frame, layout, theme, view);
    context_menu(&mut frame, layout, theme, view);
    name_prompt(&mut frame, layout, theme, view);

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

        if view.focused && view.hovered_control.is_some() {
            frame.text(
                TextRun::new(control.glyph(), rect, rect.height * 0.95, fill.shade(0.78))
                    .weight(Weight::Semibold)
                    .align(TextAlign::Center)
                    .line_height(rect.height),
            );
        }
    }

    let side = (bar.height * 0.45).round();
    let controls_end = control_rect(bar, WindowControl::Maximize).right();
    let mark = logo::mark(
        side,
        controls_end + logo::clear_space(side) + 14.0,
        bar.y + (bar.height - side) / 2.0,
    );
    logo::draw(
        frame,
        mark,
        if theme.appearance == crc_theme::Appearance::Dark {
            Brand::on_dark()
        } else {
            Brand::colour()
        },
    );

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

fn rail(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(rail) = layout.rail else { return };
    frame.quad(Quad::filled(rail, theme.chrome.panel));
    hairline_right(frame, rail, theme.chrome.border);

    let metrics = theme.metrics();

    for (index, action) in rail_view::RailAction::ALL.into_iter().enumerate() {
        let button = rail_view::button(rail, &metrics, index);
        let current = match action {
            rail_view::RailAction::Explorer => layout.sidebar.is_some(),
            rail_view::RailAction::Search => view.search.is_some(),
            rail_view::RailAction::Settings => view.settings.is_some(),
        };
        let hovered = view.hovered_rail == Some(action);

        if current || hovered {
            frame.quad(
                Quad::filled(
                    button,
                    if current {
                        theme.chrome.selected
                    } else {
                        theme.chrome.hover
                    },
                )
                .rounded(metrics.corner_radius_small),
            );
        }

        frame.text(TextRun::icon(
            action.glyph(),
            button,
            18.0,
            if current {
                theme.chrome.accent
            } else if hovered {
                theme.chrome.text
            } else {
                theme.chrome.text_faint
            },
        ));
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

    if view.search.is_some() {
        search_panel(frame, sidebar, theme, view);
        return;
    }

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

    for (index, button) in hit::ExplorerButton::ALL.into_iter().enumerate() {
        let rect = hit::explorer_button(sidebar, &metrics, index);
        let hovered = view.hovered_explorer == Some(button);

        if hovered {
            frame.quad(
                Quad::filled(rect, theme.chrome.hover).rounded(metrics.corner_radius_small),
            );
        }
        frame.text(TextRun::icon(
            button.glyph(),
            rect,
            14.0,
            if hovered {
                theme.chrome.text_strong
            } else {
                theme.chrome.text_faint
            },
        ));
    }

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
        let marker = Rect::new(row.x + indent, row.y, 16.0, row.height);
        frame.text(TextRun::icon(
            if entry.is_dir {
                icon::FOLDER
            } else {
                icon::for_name(&entry.name)
            },
            marker,
            14.0,
            if entry.is_dir {
                theme.chrome.text_muted
            } else if entry.modified {
                theme.chrome.warning
            } else {
                theme.chrome.text_faint
            },
        ));

        let text = Rect::new(
            row.x + indent + 22.0,
            row.y,
            row.width - indent - 28.0,
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
    if bar.is_empty() {
        return;
    }
    let scale = theme.type_scale;

    frame.quad(Quad::filled(bar, theme.chrome.panel));
    hairline_bottom(frame, bar, theme.chrome.border);

    for (index, rect) in tabs::rects(bar, &view.tabs, &scale).into_iter().enumerate() {
        let tab = &view.tabs[index];
        let hovered = view.hovered_tab == Some(index);

        if tab.active {
            frame.quad(Quad::filled(rect, theme.chrome.surface));
            frame.quad(Quad::filled(
                Rect::new(rect.x, rect.y, rect.width, 2.0),
                theme.chrome.accent,
            ));
        } else if hovered {
            frame.quad(Quad::filled(rect, theme.chrome.hover));
        }

        let label = Rect::new(
            rect.x + tabs::PADDING,
            rect.y,
            rect.width - tabs::PADDING * 2.0 - tabs::CLOSE_SIZE - tabs::CLOSE_GAP,
            rect.height,
        );
        frame.text(
            TextRun::new(
                tab.name.clone(),
                label,
                scale.small,
                if tab.active {
                    theme.chrome.text_strong
                } else {
                    theme.chrome.text_muted
                },
            )
            .line_height(rect.height),
        );

        let close = tabs::close_rect(rect);
        let unsaved = tab.modified || (tab.active && view.dirty);

        if unsaved && !hovered {
            frame.quad(
                Quad::filled(close.inset(4.0), theme.chrome.warning).rounded(tabs::CLOSE_SIZE),
            );
        } else if hovered || tab.active {
            if hovered {
                frame.quad(
                    Quad::filled(close, theme.chrome.selected)
                        .rounded(theme.metrics().corner_radius_small * 0.7),
                );
            }
            frame.text(
                TextRun::new(
                    "×",
                    close,
                    scale.body,
                    if hovered {
                        theme.chrome.text_strong
                    } else {
                        theme.chrome.text_faint
                    },
                )
                .align(TextAlign::Center)
                .line_height(close.height),
            );
        }

        frame.quad(Quad::filled(
            Rect::new(rect.right() - 1.0, bar.y + 6.0, 1.0, bar.height - 12.0),
            theme.chrome.border,
        ));
    }
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

    for (index, found) in view.matches.iter().enumerate() {
        let Some(local) = visible.local(found) else {
            continue;
        };
        let current = view.current_match == Some(index);

        for band in bands(&visible.text, &local) {
            if band.row >= rows {
                break;
            }
            let x = buffer.x + band.start_column as f32 * metrics.char_width;
            let width = (band.end_column - band.start_column) as f32 * metrics.char_width;
            let rect = Rect::new(
                x,
                buffer.y + band.row as f32 * metrics.line_height,
                width.min(buffer.right() - x),
                metrics.line_height,
            );

            frame.quad(if current {
                Quad::filled(rect, theme.chrome.accent_solid).rounded(2.0)
            } else {
                Quad::filled(rect, theme.chrome.accent_wash)
                    .rounded(2.0)
                    .bordered(1.0, theme.chrome.accent)
            });
        }
    }

    if let Some(selected) = view.selection.as_ref()
        && let Some(local) = visible.local(selected)
    {
        for band in bands(&visible.text, &local) {
            if band.row >= rows {
                break;
            }
            let x = buffer.x + band.start_column as f32 * metrics.char_width;
            let width = if band.to_line_end {
                (band.end_column - band.start_column) as f32 * metrics.char_width
                    + metrics.char_width * 0.6
            } else {
                (band.end_column - band.start_column) as f32 * metrics.char_width
            };
            frame.quad(Quad::filled(
                Rect::new(
                    x,
                    buffer.y + band.row as f32 * metrics.line_height,
                    width.min(buffer.right() - x),
                    metrics.line_height,
                ),
                theme.syntax.selection,
            ));
        }
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

fn panel(
    frame: &mut Frame,
    layout: &Shell,
    theme: &Theme,
    view: &EditorView,
    code: CodeMetrics,
) {
    let Some(panel) = layout.panel else { return };
    let metrics = theme.metrics();
    let scale = theme.type_scale;
    let state = &view.panel;

    frame.quad(Quad::filled(panel, theme.chrome.panel));
    frame.quad(Quad::filled(
        Rect::new(panel.x, panel.y, panel.width, 1.0),
        theme.chrome.border,
    ));

    let placed = panel_view::layout(panel, state, &metrics, scale.small * 0.62);

    for (index, tab) in panel_view::PanelTab::ALL.into_iter().enumerate() {
        let rect = placed.tabs[index];
        let current = tab == state.tab;

        if current {
            frame.quad(
                Quad::filled(rect, theme.chrome.selected).rounded(metrics.corner_radius_small),
            );
        }

        frame.text(TextRun::icon(
            tab.glyph(),
            Rect::new(rect.x + 6.0, rect.y, 14.0, rect.height),
            11.0,
            if current {
                theme.chrome.text_strong
            } else {
                theme.chrome.text_faint
            },
        ));

        let count = state.count(tab);
        let label = if count > 0 {
            format!("{} {count}", tab.title())
        } else {
            tab.title().to_string()
        };

        frame.text(
            TextRun::new(
                label,
                Rect::new(rect.x + 20.0, rect.y, rect.width - 24.0, rect.height),
                scale.small,
                if current {
                    theme.chrome.text_strong
                } else {
                    theme.chrome.text_muted
                },
            )
            .line_height(rect.height),
        );
    }

    if state.shows_a_shell() {
        shell_screen(frame, placed.body, theme, state, code);
        return;
    }

    if state.rows() == 0 {
        frame.text(
            TextRun::new(
                state.empty_note(),
                placed.body.inset_by(metrics.panel_padding, 8.0),
                scale.small,
                theme.chrome.text_faint,
            )
            .line_height(metrics.row_height),
        );
        return;
    }

    for (offset, rect) in placed.rows.iter().enumerate() {
        let index = offset + state.scroll;

        if state.selected == Some(index) {
            frame.quad(
                Quad::filled(rect.inset_by(4.0, 0.0), theme.chrome.selected)
                    .rounded(metrics.corner_radius_small),
            );
        } else if state.hovered == Some(index) {
            frame.quad(
                Quad::filled(rect.inset_by(4.0, 0.0), theme.chrome.hover)
                    .rounded(metrics.corner_radius_small),
            );
        }

        match state.tab {
            panel_view::PanelTab::Terminal => break,
            panel_view::PanelTab::Problems => {
                let Some(problem) = state.problems.get(index) else {
                    break;
                };

                frame.text(TextRun::icon(
                    icon::PROBLEMS,
                    Rect::new(rect.x + metrics.panel_padding, rect.y, 14.0, rect.height),
                    11.0,
                    theme.chrome.warning,
                ));
                frame.text(
                    TextRun::new(
                        format!("{}:{}", problem.line + 1, problem.column + 1),
                        Rect::new(rect.x + metrics.panel_padding + 18.0, rect.y, 68.0, rect.height),
                        scale.small,
                        theme.chrome.text_faint,
                    )
                    .mono()
                    .line_height(rect.height),
                );
                frame.text(
                    TextRun::new(
                        problem.message.clone(),
                        Rect::new(
                            rect.x + metrics.panel_padding + 90.0,
                            rect.y,
                            (rect.width - 240.0).max(0.0),
                            rect.height,
                        ),
                        scale.small,
                        theme.chrome.text,
                    )
                    .line_height(rect.height),
                );
                frame.text(
                    TextRun::new(
                        problem.file.clone(),
                        Rect::new(rect.x, rect.y, rect.width - metrics.panel_padding, rect.height),
                        scale.small,
                        theme.chrome.text_faint,
                    )
                    .align(TextAlign::End)
                    .line_height(rect.height),
                );
            }
            panel_view::PanelTab::Output => {
                let Some(line) = state.output.get(index) else {
                    break;
                };
                frame.text(
                    TextRun::new(
                        line.clone(),
                        Rect::new(
                            rect.x + metrics.panel_padding,
                            rect.y,
                            rect.width - metrics.panel_padding * 2.0,
                            rect.height,
                        ),
                        scale.small,
                        theme.chrome.text,
                    )
                    .mono()
                    .line_height(rect.height),
                );
            }
        }
    }
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

fn command_palette(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(state) = view.palette.as_ref() else {
        return;
    };

    let window = layout.window;
    let scale = theme.scale;
    let metrics = theme.metrics();
    let type_scale = theme.type_scale;

    frame.overlay_quad(Quad::filled(window, theme.chrome.backdrop.with_alpha(150)));

    let panel = palette::frame(window, state.rows.len(), scale);
    frame.overlay_quad(
        Quad::filled(panel, theme.chrome.raised)
            .rounded(metrics.corner_radius)
            .bordered(metrics.border_width, theme.chrome.border),
    );

    let input = palette::input_rect(panel, scale);
    frame.overlay_quad(Quad::filled(
        Rect::new(input.x, input.bottom() - 1.0, input.width, 1.0),
        theme.chrome.border,
    ));

    let caret_gap = metrics.panel_padding + 18.0 * scale;
    frame.overlay_text(
        TextRun::new(
            ">",
            Rect::new(
                input.x + metrics.panel_padding,
                input.y,
                20.0 * scale,
                input.height,
            ),
            type_scale.body,
            theme.chrome.accent,
        )
        .mono()
        .line_height(input.height),
    );

    if state.query.is_empty() {
        frame.overlay_text(
            TextRun::new(
                "Что сделать",
                Rect::new(
                    input.x + caret_gap,
                    input.y,
                    input.width - caret_gap,
                    input.height,
                ),
                type_scale.large,
                theme.chrome.text_faint,
            )
            .line_height(input.height),
        );
    } else {
        frame.overlay_text(
            TextRun::new(
                state.query.clone(),
                Rect::new(
                    input.x + caret_gap,
                    input.y,
                    input.width - caret_gap,
                    input.height,
                ),
                type_scale.large,
                theme.chrome.text_strong,
            )
            .line_height(input.height),
        );
    }

    for (index, row) in state.rows.iter().enumerate().take(palette::MAX_ROWS) {
        let rect = palette::row_rect(panel, index, scale);
        if rect.bottom() > palette::footer_rect(panel, scale).y {
            break;
        }

        if index == state.selected {
            frame.overlay_quad(
                Quad::filled(
                    rect.inset_by(PADDING_X * scale, 2.0 * scale),
                    theme.chrome.selected,
                )
                .rounded(metrics.corner_radius_small),
            );
        }

        let spans = row
            .matched
            .iter()
            .map(|range| Span::new(range.clone(), theme.chrome.accent))
            .collect();

        let label = Rect::new(
            rect.x + metrics.panel_padding,
            rect.y,
            rect.width - metrics.panel_padding * 2.0 - 90.0 * scale,
            rect.height,
        );
        frame.overlay_text(
            TextRun::new(
                row.title.clone(),
                label,
                type_scale.body,
                if index == state.selected {
                    theme.chrome.text_strong
                } else {
                    theme.chrome.text
                },
            )
            .line_height(rect.height)
            .spans(spans),
        );

        if let Some(hint) = row.hint.as_ref() {
            frame.overlay_text(
                TextRun::new(
                    hint.clone(),
                    Rect::new(
                        rect.x + metrics.panel_padding,
                        rect.y,
                        rect.width - metrics.panel_padding * 2.0,
                        rect.height,
                    ),
                    type_scale.small,
                    theme.chrome.text_faint,
                )
                .mono()
                .align(TextAlign::End)
                .line_height(rect.height),
            );
        }
    }

    if state.rows.is_empty() && !state.query.is_empty() {
        let empty = palette::row_rect(panel, 0, scale);
        frame.overlay_text(
            TextRun::new(
                "Ничего не нашлось",
                Rect::new(
                    empty.x + metrics.panel_padding,
                    empty.y,
                    empty.width,
                    empty.height,
                ),
                type_scale.body,
                theme.chrome.text_faint,
            )
            .line_height(empty.height),
        );
    }

    let footer = palette::footer_rect(panel, scale);
    frame.overlay_quad(Quad::filled(
        Rect::new(footer.x, footer.y, footer.width, 1.0),
        theme.chrome.border,
    ));
    frame.overlay_text(
        TextRun::new(
            "стрелки — выбрать    ввод — выполнить    esc — закрыть",
            Rect::new(
                footer.x + metrics.panel_padding,
                footer.y,
                footer.width - metrics.panel_padding * 2.0,
                footer.height,
            ),
            type_scale.small,
            theme.chrome.text_faint,
        )
        .mono()
        .line_height(footer.height),
    );
}

const PADDING_X: f32 = 6.0;

fn welcome_screen(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(state) = view.welcome.as_ref() else {
        return;
    };

    let scale = theme.scale;
    let metrics = theme.metrics();
    let type_scale = theme.type_scale;

    let window = Rect::new(
        layout.window.x,
        layout.titlebar.bottom(),
        layout.window.width,
        layout.window.bottom() - layout.titlebar.bottom(),
    );
    frame.quad(Quad::filled(window, theme.chrome.surface));

    let placed = welcome::layout(window, state, scale);

    let brand = if theme.appearance == crc_theme::Appearance::Dark {
        Brand::on_dark()
    } else {
        Brand::colour()
    };
    logo::draw(
        frame,
        logo::mark(placed.mark.width, placed.mark.x, placed.mark.y),
        brand,
    );

    frame.text(
        TextRun::new(
            "CRC Code",
            placed.title,
            type_scale.display * 1.05,
            theme.chrome.text_strong,
        )
        .weight(Weight::Semibold)
        .line_height(placed.title.height),
    );
    frame.text(
        TextRun::new(
            "Тихий редактор, который не мешает думать.",
            placed.tagline,
            type_scale.large,
            theme.chrome.text_muted,
        )
        .line_height(placed.tagline.height),
    );

    if !placed.recent.is_empty() {
        frame.text(
            TextRun::new(
                "Недавние проекты",
                placed.recent_heading,
                type_scale.small,
                theme.chrome.text_faint,
            )
            .weight(Weight::Semibold)
            .line_height(placed.recent_heading.height),
        );
    }

    for (index, rect) in placed.recent.iter().enumerate() {
        let Some(entry) = state.recent.get(index) else {
            break;
        };
        let hovered = state.hovered == Some(welcome::Target::Recent(index));

        if hovered {
            frame.quad(
                Quad::filled(rect.inset_by(-8.0 * scale, 0.0), theme.chrome.hover)
                    .rounded(metrics.corner_radius_small),
            );
        }

        frame.text(
            TextRun::new(
                entry.name.clone(),
                Rect::new(
                    rect.x,
                    rect.y + 4.0 * scale,
                    rect.width * 0.55,
                    rect.height * 0.5,
                ),
                type_scale.large,
                theme.chrome.text_strong,
            )
            .line_height(rect.height * 0.5),
        );
        frame.text(
            TextRun::new(
                entry.path.clone(),
                Rect::new(
                    rect.x,
                    rect.y + rect.height * 0.5,
                    rect.width * 0.75,
                    rect.height * 0.45,
                ),
                type_scale.small,
                theme.chrome.text_faint,
            )
            .mono()
            .line_height(rect.height * 0.45),
        );
        frame.text(
            TextRun::new(
                entry.when.clone(),
                Rect::new(rect.x, rect.y, rect.width, rect.height),
                type_scale.small,
                theme.chrome.text_faint,
            )
            .align(TextAlign::End)
            .line_height(rect.height),
        );
    }

    let opening = state.hovered == Some(welcome::Target::OpenFolder);
    frame.quad(
        Quad::filled(
            placed.open_folder,
            if opening {
                theme.chrome.accent_hover
            } else {
                theme.chrome.accent_solid
            },
        )
        .rounded(metrics.corner_radius_small),
    );
    frame.text(
        TextRun::new(
            "Открыть папку",
            placed.open_folder,
            type_scale.body,
            theme.chrome.text_on_accent,
        )
        .weight(Weight::Semibold)
        .align(TextAlign::Center)
        .line_height(placed.open_folder.height),
    );

    for (rect, (keys, what)) in placed.hints.iter().zip(state.hints.iter()) {
        frame.text(
            TextRun::new(
                keys.clone(),
                Rect::new(rect.x, rect.y, 110.0 * scale, rect.height),
                type_scale.small,
                theme.chrome.accent,
            )
            .mono()
            .line_height(rect.height),
        );
        frame.text(
            TextRun::new(
                what.clone(),
                Rect::new(
                    rect.x + 118.0 * scale,
                    rect.y,
                    rect.width - 118.0 * scale,
                    rect.height,
                ),
                type_scale.small,
                theme.chrome.text_muted,
            )
            .line_height(rect.height),
        );
    }
}

fn settings_panel(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(state) = view.settings.as_ref() else {
        return;
    };

    let scale = theme.scale;
    let metrics = theme.metrics();
    let type_scale = theme.type_scale;

    frame.overlay_quad(Quad::filled(
        layout.window,
        theme.chrome.backdrop.with_alpha(160),
    ));

    let placed = settings_view::layout(layout.window, state, scale);
    frame.overlay_quad(
        Quad::filled(placed.panel, theme.chrome.raised)
            .rounded(metrics.corner_radius)
            .bordered(metrics.border_width, theme.chrome.border),
    );

    frame.overlay_text(
        TextRun::new(
            "Настройки",
            placed.header.inset_by(settings_view::PADDING * scale, 0.0),
            type_scale.title,
            theme.chrome.text_strong,
        )
        .weight(Weight::Semibold)
        .line_height(placed.header.height),
    );
    frame.overlay_quad(Quad::filled(
        Rect::new(
            placed.header.x,
            placed.header.bottom() - 1.0,
            placed.header.width,
            1.0,
        ),
        theme.chrome.border,
    ));
    frame.overlay_text(
        TextRun::new(
            "\u{00d7}",
            placed.close,
            type_scale.large,
            if state.hovered == Some(settings_view::Target::Close) {
                theme.chrome.text_strong
            } else {
                theme.chrome.text_faint
            },
        )
        .align(TextAlign::Center)
        .line_height(placed.close.height),
    );

    frame.overlay_quad(Quad::filled(placed.sidebar, theme.chrome.panel));
    frame.overlay_quad(Quad::filled(
        Rect::new(
            placed.sidebar.right() - 1.0,
            placed.sidebar.y,
            1.0,
            placed.sidebar.height,
        ),
        theme.chrome.border,
    ));

    for (index, rect) in placed.sections.iter().enumerate() {
        let section = settings_view::Section::ALL[index];
        let current = section == state.section;

        if current {
            frame.overlay_quad(
                Quad::filled(*rect, theme.chrome.selected).rounded(metrics.corner_radius_small),
            );
        } else if state.hovered == Some(settings_view::Target::Section(index)) {
            frame.overlay_quad(
                Quad::filled(*rect, theme.chrome.hover).rounded(metrics.corner_radius_small),
            );
        }

        frame.overlay_text(
            TextRun::new(
                section.title(),
                rect.inset_by(12.0 * scale, 0.0),
                type_scale.body,
                if current {
                    theme.chrome.text_strong
                } else {
                    theme.chrome.text
                },
            )
            .line_height(rect.height),
        );
    }

    if let Some(thumb) = placed.thumb {
        frame.overlay_quad(Quad::filled(thumb, theme.chrome.border).rounded(thumb.width / 2.0));
    }

    if let Some(field) = placed.search {
        frame.overlay_quad(
            Quad::filled(field, theme.chrome.surface)
                .rounded(metrics.corner_radius_small)
                .bordered(
                    metrics.border_width,
                    if state.hovered == Some(settings_view::Target::Search) {
                        theme.chrome.accent
                    } else {
                        theme.chrome.border
                    },
                ),
        );

        let empty = state.query.is_empty();
        frame.overlay_text(
            TextRun::new(
                if empty {
                    "Поиск по командам".to_string()
                } else {
                    state.query.clone()
                },
                field.inset_by(12.0 * scale, 0.0),
                type_scale.body,
                if empty {
                    theme.chrome.text_faint
                } else {
                    theme.chrome.text_strong
                },
            )
            .line_height(field.height),
        );
    }

    if let Some(button) = placed.reset {
        let live = state.touched();
        let hovered = live && state.hovered == Some(settings_view::Target::Reset);

        frame.overlay_quad(
            Quad::filled(
                button,
                if hovered {
                    theme.chrome.hover
                } else {
                    theme.chrome.panel
                },
            )
            .rounded(metrics.corner_radius_small)
            .bordered(metrics.border_width, theme.chrome.border),
        );
        frame.overlay_text(
            TextRun::new(
                "Сбросить",
                button,
                type_scale.small,
                if live {
                    theme.chrome.text
                } else {
                    theme.chrome.text_faint
                },
            )
            .align(TextAlign::Center)
            .line_height(button.height),
        );
    }

    let shown = state.shown();
    if shown.is_empty() {
        frame.overlay_text(
            TextRun::new(
                "Ничего не нашлось",
                placed.body,
                type_scale.body,
                theme.chrome.text_faint,
            )
            .align(TextAlign::Center)
            .line_height(placed.body.height),
        );
        return;
    }

    match state.section {
        settings_view::Section::Appearance => {
            for (offset, rect) in placed.rows.iter().enumerate() {
                let Some(toggle) = shown
                    .get(offset + state.scroll)
                    .and_then(|index| state.toggles.get(*index))
                else {
                    break;
                };
                let index = shown[offset + state.scroll];

                let hovered = state.hovered == Some(settings_view::Target::Toggle(index));
                if hovered {
                    frame.overlay_quad(
                        Quad::filled(*rect, theme.chrome.hover)
                            .rounded(metrics.corner_radius_small),
                    );
                }

                frame.overlay_text(
                    TextRun::new(
                        toggle.label.clone(),
                        Rect::new(
                            rect.x + 8.0 * scale,
                            rect.y + 6.0 * scale,
                            rect.width * 0.7,
                            rect.height * 0.5,
                        ),
                        type_scale.body,
                        theme.chrome.text_strong,
                    )
                    .line_height(rect.height * 0.5),
                );
                frame.overlay_text(
                    TextRun::new(
                        toggle.note.clone(),
                        Rect::new(
                            rect.x + 8.0 * scale,
                            rect.y + rect.height * 0.52,
                            rect.width * 0.7,
                            rect.height * 0.4,
                        ),
                        type_scale.small,
                        theme.chrome.text_faint,
                    )
                    .line_height(rect.height * 0.4),
                );

                let track = Rect::new(
                    rect.right() - 46.0 * scale,
                    rect.y + (rect.height - 20.0 * scale) / 2.0,
                    40.0 * scale,
                    20.0 * scale,
                );
                let switch = if toggle.on {
                    Quad::filled(track, theme.chrome.accent_solid)
                } else {
                    Quad::filled(track, theme.chrome.surface).bordered(
                        metrics.border_width,
                        if hovered {
                            theme.chrome.text
                        } else {
                            theme.chrome.text_faint
                        },
                    )
                };
                frame.overlay_quad(switch.rounded(track.height / 2.0));

                let knob = if hovered { 14.0 } else { 12.0 } * scale;
                let centre = if toggle.on {
                    track.right() - 10.0 * scale
                } else {
                    track.x + 10.0 * scale
                };
                frame.overlay_quad(
                    Quad::filled(
                        Rect::new(
                            centre - knob / 2.0,
                            track.y + (track.height - knob) / 2.0,
                            knob,
                            knob,
                        ),
                        if toggle.on {
                            theme.chrome.raised
                        } else {
                            theme.chrome.text
                        },
                    )
                    .rounded(knob / 2.0),
                );
            }
        }
        settings_view::Section::Keys => {
            for (offset, rect) in placed.rows.iter().enumerate() {
                let Some(binding) = shown
                    .get(offset + state.scroll)
                    .and_then(|index| state.bindings.get(*index))
                else {
                    break;
                };
                let index = shown[offset + state.scroll];
                let listening = state.capturing == Some(index);

                if listening {
                    frame.overlay_quad(
                        Quad::filled(*rect, theme.chrome.accent_wash)
                            .rounded(metrics.corner_radius_small),
                    );
                } else if state.hovered == Some(settings_view::Target::Binding(index)) {
                    frame.overlay_quad(
                        Quad::filled(*rect, theme.chrome.hover)
                            .rounded(metrics.corner_radius_small),
                    );
                }

                frame.overlay_text(
                    TextRun::new(
                        binding.title.clone(),
                        Rect::new(
                            rect.x + 8.0 * scale,
                            rect.y + 6.0 * scale,
                            rect.width - settings_view::KEYCAP * scale,
                            rect.height * 0.5,
                        ),
                        type_scale.body,
                        theme.chrome.text_strong,
                    )
                    .line_height(rect.height * 0.5),
                );

                let note = if listening {
                    "нажми сочетание · Esc — отмена".to_string()
                } else if let Some(other) = binding.clash.as_ref() {
                    format!("занято: {other}")
                } else if binding.changed {
                    "изменено".to_string()
                } else {
                    String::new()
                };
                if !note.is_empty() {
                    frame.overlay_text(
                        TextRun::new(
                            note,
                            Rect::new(
                                rect.x + 8.0 * scale,
                                rect.y + rect.height * 0.52,
                                rect.width - settings_view::KEYCAP * scale,
                                rect.height * 0.4,
                            ),
                            type_scale.small,
                            if binding.clash.is_some() {
                                theme.chrome.danger
                            } else {
                                theme.chrome.text_faint
                            },
                        )
                        .line_height(rect.height * 0.4),
                    );
                }

                let cap = Rect::new(
                    rect.right() - settings_view::KEYCAP * scale,
                    rect.y + (rect.height - 28.0 * scale) / 2.0,
                    (settings_view::KEYCAP - 10.0) * scale,
                    28.0 * scale,
                );
                frame.overlay_quad(
                    Quad::filled(cap, theme.chrome.surface)
                        .rounded(metrics.corner_radius_small)
                        .bordered(
                            metrics.border_width,
                            if listening {
                                theme.chrome.accent
                            } else if binding.clash.is_some() {
                                theme.chrome.danger
                            } else {
                                theme.chrome.border
                            },
                        ),
                );
                frame.overlay_text(
                    TextRun::new(
                        if binding.keys.is_empty() {
                            "не назначено".to_string()
                        } else {
                            binding.keys.clone()
                        },
                        cap,
                        type_scale.small,
                        if binding.keys.is_empty() {
                            theme.chrome.text_faint
                        } else {
                            theme.chrome.text
                        },
                    )
                    .mono()
                    .align(TextAlign::Center)
                    .line_height(cap.height),
                );
            }
        }
    }
}

fn find_bar(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(state) = view.find.as_ref() else {
        return;
    };

    let metrics = theme.metrics();
    let type_scale = theme.type_scale;
    let placed = find_view::layout(layout.buffer, theme.scale);

    frame.overlay_quad(
        Quad::filled(placed.bar, theme.chrome.raised)
            .rounded(metrics.corner_radius)
            .bordered(metrics.border_width, theme.chrome.border),
    );

    frame.overlay_quad(
        Quad::filled(placed.field, theme.chrome.surface)
            .rounded(metrics.corner_radius_small)
            .bordered(metrics.border_width, theme.chrome.accent),
    );

    let empty = state.query.is_empty();
    frame.overlay_text(
        TextRun::new(
            if empty {
                "Найти в файле".to_string()
            } else {
                state.query.clone()
            },
            placed.field.inset_by(10.0 * theme.scale, 0.0),
            type_scale.body,
            if empty {
                theme.chrome.text_faint
            } else {
                theme.chrome.text_strong
            },
        )
        .line_height(placed.field.height),
    );

    frame.overlay_text(
        TextRun::new(
            state.tally(),
            placed.tally,
            type_scale.small,
            if state.total == 0 && !empty {
                theme.chrome.danger
            } else {
                theme.chrome.text_faint
            },
        )
        .align(TextAlign::End)
        .line_height(placed.tally.height),
    );

    for target in [
        find_view::Target::MatchCase,
        find_view::Target::Previous,
        find_view::Target::Next,
        find_view::Target::Close,
    ] {
        let rect = match target {
            find_view::Target::MatchCase => placed.match_case,
            find_view::Target::Previous => placed.previous,
            find_view::Target::Next => placed.next,
            find_view::Target::Close => placed.close,
            find_view::Target::Field => continue,
        };
        let on = target == find_view::Target::MatchCase && state.match_case;
        let hovered = state.hovered == Some(target);

        if on || hovered {
            frame.overlay_quad(
                Quad::filled(
                    rect,
                    if on {
                        theme.chrome.selected
                    } else {
                        theme.chrome.hover
                    },
                )
                .rounded(metrics.corner_radius_small),
            );
        }

        frame.overlay_text(TextRun::icon(
            find_view::glyph(target),
            rect,
            14.0,
            if on {
                theme.chrome.accent
            } else if hovered {
                theme.chrome.text_strong
            } else {
                theme.chrome.text_faint
            },
        ));
    }
}

fn search_panel(frame: &mut Frame, sidebar: Rect, theme: &Theme, view: &EditorView) {
    let Some(state) = view.search.as_ref() else {
        return;
    };

    let metrics = theme.metrics();
    let type_scale = theme.type_scale;
    let placed = search_view::layout(sidebar, state, &metrics);

    frame.text(
        TextRun::new(
            "Поиск по проекту",
            placed.header.inset_by(metrics.panel_padding, 0.0),
            type_scale.small,
            theme.chrome.text_muted,
        )
        .weight(Weight::Semibold)
        .line_height(placed.header.height),
    );

    frame.quad(
        Quad::filled(placed.field, theme.chrome.surface)
            .rounded(metrics.corner_radius_small)
            .bordered(metrics.border_width, theme.chrome.accent),
    );

    let empty = state.query.is_empty();
    frame.text(
        TextRun::new(
            if empty {
                "Что искать".to_string()
            } else {
                state.query.clone()
            },
            placed.field.inset_by(8.0, 0.0),
            type_scale.body,
            if empty {
                theme.chrome.text_faint
            } else {
                theme.chrome.text_strong
            },
        )
        .line_height(placed.field.height),
    );

    if state.match_case {
        frame.quad(
            Quad::filled(placed.match_case, theme.chrome.selected)
                .rounded(metrics.corner_radius_small),
        );
    }
    frame.text(TextRun::icon(
        icon::MATCH_CASE,
        placed.match_case,
        13.0,
        if state.match_case {
            theme.chrome.accent
        } else {
            theme.chrome.text_faint
        },
    ));

    frame.text(
        TextRun::new(
            state.tally(),
            placed.tally,
            type_scale.small,
            if state.searched && state.hits == 0 && !empty {
                theme.chrome.danger
            } else {
                theme.chrome.text_faint
            },
        )
        .line_height(placed.tally.height),
    );

    for (offset, rect) in placed.rows.iter().enumerate() {
        let index = offset + state.scroll;
        let Some(row) = state.rows.get(index) else {
            break;
        };

        if state.selected == Some(index) {
            frame.quad(
                Quad::filled(rect.inset_by(4.0, 1.0), theme.chrome.selected)
                    .rounded(metrics.corner_radius_small),
            );
        } else if state.hovered == Some(index) {
            frame.quad(
                Quad::filled(rect.inset_by(4.0, 1.0), theme.chrome.hover)
                    .rounded(metrics.corner_radius_small),
            );
        }

        match row {
            search_view::SearchRow::File { path, hits } => {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.to_string_lossy().into_owned());

                frame.text(TextRun::icon(
                    icon::for_name(&name),
                    Rect::new(rect.x + 8.0, rect.y, 16.0, rect.height),
                    12.0,
                    theme.chrome.text_faint,
                ));
                frame.text(
                    TextRun::new(
                        name,
                        Rect::new(rect.x + 26.0, rect.y, rect.width - 60.0, rect.height),
                        type_scale.small,
                        theme.chrome.text_strong,
                    )
                    .weight(Weight::Semibold)
                    .line_height(rect.height),
                );
                frame.text(
                    TextRun::new(
                        hits.to_string(),
                        Rect::new(rect.x, rect.y, rect.width - 10.0, rect.height),
                        type_scale.small,
                        theme.chrome.text_faint,
                    )
                    .align(TextAlign::End)
                    .line_height(rect.height),
                );
            }
            search_view::SearchRow::Line { line, text, .. } => {
                frame.text(
                    TextRun::new(
                        line.to_string(),
                        Rect::new(rect.x + 20.0, rect.y, 32.0, rect.height),
                        type_scale.small,
                        theme.chrome.text_faint,
                    )
                    .align(TextAlign::End)
                    .mono()
                    .line_height(rect.height),
                );
                frame.text(
                    TextRun::new(
                        text.trim_start().to_string(),
                        Rect::new(rect.x + 58.0, rect.y, rect.width - 64.0, rect.height),
                        type_scale.small,
                        theme.chrome.text,
                    )
                    .mono()
                    .line_height(rect.height),
                );
            }
        }
    }
}

fn context_menu(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(state) = view.menu.as_ref() else {
        return;
    };

    let metrics = theme.metrics();
    let type_scale = theme.type_scale;
    let placed = menu_view::layout(layout.window, state, theme.scale);

    frame.overlay_quad(
        Quad::filled(placed.panel, theme.chrome.raised)
            .rounded(metrics.corner_radius)
            .bordered(metrics.border_width, theme.chrome.border),
    );

    for (index, item) in state.items.iter().enumerate() {
        let rect = placed.rows[index];

        match item {
            menu_view::MenuItem::Separator => {
                frame.overlay_quad(Quad::filled(
                    Rect::new(
                        rect.x + 10.0 * theme.scale,
                        rect.y + rect.height / 2.0,
                        rect.width - 20.0 * theme.scale,
                        1.0,
                    ),
                    theme.chrome.border,
                ));
            }
            menu_view::MenuItem::Action(action) => {
                let hovered = state.hovered == Some(index);
                if hovered {
                    frame.overlay_quad(
                        Quad::filled(
                            rect.inset_by(4.0 * theme.scale, 1.0),
                            theme.chrome.hover,
                        )
                        .rounded(metrics.corner_radius_small),
                    );
                }

                let tint = if action.destructive() {
                    theme.chrome.danger
                } else if hovered {
                    theme.chrome.text_strong
                } else {
                    theme.chrome.text
                };

                frame.overlay_text(TextRun::icon(
                    action.glyph(),
                    Rect::new(rect.x + 12.0 * theme.scale, rect.y, 16.0, rect.height),
                    12.0,
                    tint,
                ));
                frame.overlay_text(
                    TextRun::new(
                        action.title(),
                        Rect::new(
                            rect.x + 36.0 * theme.scale,
                            rect.y,
                            rect.width - 44.0 * theme.scale,
                            rect.height,
                        ),
                        type_scale.body,
                        tint,
                    )
                    .line_height(rect.height),
                );
            }
        }
    }
}

fn name_prompt(frame: &mut Frame, layout: &Shell, theme: &Theme, view: &EditorView) {
    let Some(state) = view.prompt.as_ref() else {
        return;
    };

    let metrics = theme.metrics();
    let type_scale = theme.type_scale;
    let placed = prompt_view::layout(layout.window, state, theme.scale);

    frame.overlay_quad(Quad::filled(
        layout.window,
        theme.chrome.backdrop.with_alpha(150),
    ));
    frame.overlay_quad(
        Quad::filled(placed.panel, theme.chrome.raised)
            .rounded(metrics.corner_radius)
            .bordered(metrics.border_width, theme.chrome.border),
    );

    frame.overlay_text(
        TextRun::new(
            state.kind.title(),
            placed.title,
            type_scale.large,
            theme.chrome.text_strong,
        )
        .weight(Weight::Semibold)
        .line_height(placed.title.height),
    );
    frame.overlay_text(
        TextRun::new(
            state.complaint.clone().unwrap_or_else(|| state.note.clone()),
            placed.note,
            type_scale.small,
            if state.complaint.is_some() {
                theme.chrome.danger
            } else {
                theme.chrome.text_faint
            },
        )
        .line_height(placed.note.height),
    );

    if state.kind.asks_for_a_name() {
        frame.overlay_quad(
            Quad::filled(placed.field, theme.chrome.surface)
                .rounded(metrics.corner_radius_small)
                .bordered(metrics.border_width, theme.chrome.accent),
        );

        let empty = state.value.is_empty();
        frame.overlay_text(
            TextRun::new(
                if empty {
                    "имя".to_string()
                } else {
                    state.value.clone()
                },
                placed.field.inset_by(12.0 * theme.scale, 0.0),
                type_scale.body,
                if empty {
                    theme.chrome.text_faint
                } else {
                    theme.chrome.text_strong
                },
            )
            .line_height(placed.field.height),
        );
    }

    for (target, rect) in [
        (prompt_view::Target::Cancel, placed.cancel),
        (prompt_view::Target::Confirm, placed.confirm),
    ] {
        let primary = target == prompt_view::Target::Confirm;
        let live = !primary || state.ready();
        let hovered = state.hovered == Some(target);

        let fill = if primary && live {
            if state.kind.destructive() {
                theme.chrome.danger
            } else if hovered {
                theme.chrome.accent_hover
            } else {
                theme.chrome.accent_solid
            }
        } else if hovered {
            theme.chrome.hover
        } else {
            theme.chrome.panel
        };

        frame.overlay_quad(
            Quad::filled(rect, fill)
                .rounded(metrics.corner_radius_small)
                .bordered(metrics.border_width, theme.chrome.border),
        );
        frame.overlay_text(
            TextRun::new(
                if primary {
                    state.kind.confirm()
                } else {
                    "Отмена"
                },
                rect,
                type_scale.body,
                if primary && live {
                    theme.chrome.text_on_accent
                } else if live {
                    theme.chrome.text
                } else {
                    theme.chrome.text_faint
                },
            )
            .align(TextAlign::Center)
            .line_height(rect.height),
        );
    }
}

fn shell_screen(
    frame: &mut Frame,
    body: Rect,
    theme: &Theme,
    state: &panel_view::PanelView,
    code: CodeMetrics,
) {
    let palette = &theme.terminal;
    frame.quad(Quad::filled(body, palette.background));

    if state.focused {
        frame.quad(Quad::filled(
            Rect::new(body.x, body.y, 2.0, body.height),
            theme.chrome.accent,
        ));
    }

    let Some(screen) = state.screen.as_ref() else {
        frame.text(
            TextRun::new(
                state.empty_note(),
                body.inset_by(10.0, 8.0),
                theme.type_scale.small,
                theme.chrome.text_faint,
            )
            .mono()
            .line_height(code.line_height),
        );
        return;
    };

    let width = code.char_width.max(1.0);
    let height = code.line_height.max(1.0);

    for (row, line) in screen.rows.iter().enumerate() {
        let top = body.y + row as f32 * height;
        if top + height > body.bottom() + 0.5 {
            break;
        }

        for (column, cell) in line.cells.iter().enumerate() {
            let back = paint(palette, cell.background, cell.inverse, true);
            if let Some(colour) = back {
                frame.quad(Quad::filled(
                    Rect::new(body.x + column as f32 * width, top, width, height),
                    colour,
                ));
            }
        }

        let text = line.text();
        if text.trim().is_empty() {
            continue;
        }

        let mut spans: Vec<Span> = Vec::new();
        let mut at = 0usize;
        for cell in &line.cells {
            let len = cell.text.len();
            let colour = paint(palette, cell.foreground, cell.inverse, false)
                .unwrap_or(palette.foreground);

            match spans.last_mut() {
                Some(last) if last.color == colour => last.range.end = at + len,
                _ => spans.push(Span::new(at..at + len, colour)),
            }
            at += len;
        }

        if spans.len() == 1 && spans[0].color == palette.foreground {
            spans.clear();
        }

        frame.text(
            TextRun::new(
                text,
                Rect::new(body.x, top, body.width, height),
                theme.type_scale.code,
                palette.foreground,
            )
            .mono()
            .line_height(height)
            .spans(spans),
        );
    }

    if screen.cursor_visible && screen.alive {
        let (row, column) = screen.cursor;
        let caret = Rect::new(
            body.x + column as f32 * width,
            body.y + row as f32 * height,
            2.0,
            height,
        );
        if body.contains(caret.x, caret.y) {
            frame.quad(Quad::filled(caret, palette.cursor));
        }
    }
}

fn paint(
    palette: &crc_theme::TerminalTheme,
    ink: crc_term::screen::Ink,
    inverse: bool,
    background: bool,
) -> Option<crc_theme::Rgba> {
    let wanted = if inverse { !background } else { background };

    let colour = match ink {
        crc_term::screen::Ink::Default => {
            if wanted {
                return inverse.then_some(palette.foreground);
            }
            palette.foreground
        }
        crc_term::screen::Ink::Indexed(index) => palette.colour(index),
        crc_term::screen::Ink::Rgb(r, g, b) => crc_theme::Rgba::new(r, g, b, 255),
    };

    if wanted && matches!(ink, crc_term::screen::Ink::Default) && !inverse {
        return None;
    }
    Some(colour)
}
