use crc_theme::{Density, Theme};
use crc_ui::geometry::Rect;
use crc_ui::{Shell, ShellState};

fn window() -> Rect {
    Rect::from_size(1440.0, 900.0)
}

#[test]
fn the_shell_covers_the_window_top_to_bottom() {
    let shell = Shell::compute(window(), &Theme::light(), &ShellState::default());

    assert_eq!(shell.titlebar.y, 0.0);
    let statusbar = shell.statusbar.expect("a status bar");
    assert_eq!(statusbar.bottom(), 900.0);
    assert!(shell.titlebar.height + shell.buffer.height + statusbar.height <= 900.0);
}

#[test]
fn nothing_on_screen_overlaps_anything_else() {
    let state = ShellState {
        aside_open: true,
        ..ShellState::default()
    };
    let shell = Shell::compute(window(), &Theme::light(), &state);
    let regions = shell.regions();

    for (i, (name_a, a)) in regions.iter().enumerate() {
        for (name_b, b) in regions.iter().skip(i + 1) {
            assert!(
                a.intersection(b).is_none(),
                "{name_a} {a:?} overlaps {name_b} {b:?}"
            );
        }
    }
}

#[test]
fn every_region_stays_inside_the_window() {
    let state = ShellState {
        aside_open: true,
        ..ShellState::default()
    };
    let shell = Shell::compute(window(), &Theme::light(), &state);

    for (name, rect) in shell.regions() {
        assert!(rect.x >= 0.0 && rect.y >= 0.0, "{name} starts outside");
        assert!(
            rect.right() <= 1440.0 && rect.bottom() <= 900.0,
            "{name} runs past the window: {rect:?}"
        );
    }
}

#[test]
fn the_gutter_sits_immediately_left_of_the_buffer() {
    let shell = Shell::compute(window(), &Theme::light(), &ShellState::default());

    assert_eq!(shell.gutter.right(), shell.buffer.x);
    assert_eq!(shell.gutter.y, shell.buffer.y);
    assert_eq!(shell.gutter.height, shell.buffer.height);
}

#[test]
fn closing_the_sidebar_gives_its_width_to_the_buffer() {
    let theme = Theme::light();
    let open = Shell::compute(window(), &theme, &ShellState::default());
    let closed = Shell::compute(
        window(),
        &theme,
        &ShellState {
            sidebar_open: false,
            ..ShellState::default()
        },
    );

    assert!(closed.sidebar.is_none());
    let sidebar_width = open.sidebar.expect("a sidebar").width;
    assert_eq!(closed.buffer.width, open.buffer.width + sidebar_width);
}

#[test]
fn the_calm_profile_leaves_only_the_essentials() {
    let theme = Theme::light().with_density(Density::Calm);
    let shell = Shell::compute(window(), &theme, &ShellState::default());

    assert!(shell.rail.is_none(), "calm has no activity rail");
    assert!(shell.minimap.is_none());
    assert!(shell.panel.is_none());
    assert!(shell.breadcrumbs.is_none());
    assert!(shell.sidebar.is_some(), "but it keeps one sidebar");
}

#[test]
fn the_dense_profile_shows_everything() {
    let theme = Theme::light().with_density(Density::Dense);
    let shell = Shell::compute(window(), &theme, &ShellState::default());

    assert!(shell.rail.is_some());
    assert!(shell.minimap.is_some());
    assert!(shell.panel.is_some());
    assert!(shell.breadcrumbs.is_some());
}

#[test]
fn zen_leaves_the_code_and_the_title_bar() {
    for density in [Density::Calm, Density::Balanced, Density::Dense] {
        let mut theme = Theme::light().with_density(density);
        theme.zen = true;
        let shell = Shell::compute(window(), &theme, &ShellState::default());

        assert!(shell.sidebar.is_none(), "{density:?}");
        assert!(shell.rail.is_none(), "{density:?}");
        assert!(shell.minimap.is_none(), "{density:?}");
        assert!(shell.panel.is_none(), "{density:?}");
        assert!(shell.aside.is_none(), "{density:?}");
        assert!(shell.statusbar.is_none(), "{density:?}");
        assert!(!shell.titlebar.is_empty(), "{density:?}");
        assert!(shell.buffer.width > 1200.0, "{density:?}");
    }
}

#[test]
fn zen_gives_the_buffer_almost_the_whole_window() {
    let plain = Shell::compute(window(), &Theme::light(), &ShellState::default());
    let mut zen_theme = Theme::light();
    zen_theme.zen = true;
    let zen = Shell::compute(window(), &zen_theme, &ShellState::default());

    assert!(zen.buffer.width > plain.buffer.width * 1.3);
    assert!(zen.buffer.height > plain.buffer.height);
}

#[test]
fn a_narrow_window_drops_panes_rather_than_squeezing_the_code() {
    let narrow = Rect::from_size(420.0, 700.0);
    let shell = Shell::compute(narrow, &Theme::light(), &ShellState::default());

    assert!(
        shell.sidebar.is_none(),
        "the sidebar would leave the buffer unusable"
    );
    assert!(
        shell.buffer.width >= 200.0,
        "the buffer kept a workable width: {:?}",
        shell.buffer
    );
}

#[test]
fn a_short_window_drops_the_bottom_panel() {
    let short = Rect::from_size(1440.0, 300.0);
    let shell = Shell::compute(short, &Theme::light(), &ShellState::default());

    assert!(shell.panel.is_none());
    assert!(shell.buffer.height > 0.0);
}

#[test]
fn a_window_shrunk_to_nothing_still_produces_valid_rects() {
    for size in [(0.0, 0.0), (1.0, 1.0), (40.0, 20.0), (200.0, 60.0)] {
        let shell = Shell::compute(
            Rect::from_size(size.0, size.1),
            &Theme::light(),
            &ShellState::default(),
        );

        for (name, rect) in shell.regions() {
            assert!(
                rect.width >= 0.0 && rect.height >= 0.0,
                "{name} went negative at {size:?}: {rect:?}"
            );
            assert!(
                rect.right() <= size.0 + 0.001 && rect.bottom() <= size.1 + 0.001,
                "{name} escaped the {size:?} window: {rect:?}"
            );
        }
    }
}

#[test]
fn the_aside_opens_on_the_right() {
    let state = ShellState {
        aside_open: true,
        aside_width: 320.0,
        ..ShellState::default()
    };
    let shell = Shell::compute(window(), &Theme::light(), &state);

    let aside = shell.aside.expect("the preview column");
    assert_eq!(aside.right(), 1440.0);
    assert_eq!(aside.width, 320.0);
    assert!(shell.buffer.right() <= aside.x);
}

#[test]
fn a_point_resolves_to_the_region_under_it() {
    let shell = Shell::compute(window(), &Theme::light(), &ShellState::default());

    assert_eq!(shell.region_at(700.0, 10.0), Some("titlebar"));
    assert_eq!(
        shell.region_at(shell.buffer.x + 10.0, shell.buffer.y + 10.0),
        Some("buffer")
    );
    let sidebar = shell.sidebar.expect("a sidebar");
    assert_eq!(
        shell.region_at(sidebar.x + 5.0, sidebar.y + 5.0),
        Some("sidebar")
    );
    assert_eq!(shell.region_at(-1.0, -1.0), None);
}

mod rects {
    use super::*;

    #[test]
    fn a_cut_adds_back_up_to_the_whole() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let (top, rest) = rect.split_top(15.0);

        assert_eq!(top, Rect::new(10.0, 20.0, 100.0, 15.0));
        assert_eq!(rest, Rect::new(10.0, 35.0, 100.0, 35.0));
        assert_eq!(top.height + rest.height, rect.height);
    }

    #[test]
    fn cutting_from_each_side_lands_where_it_should() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);

        assert_eq!(rect.split_bottom(20.0).0, Rect::new(0.0, 80.0, 100.0, 20.0));
        assert_eq!(rect.split_left(30.0).0, Rect::new(0.0, 0.0, 30.0, 100.0));
        assert_eq!(rect.split_right(30.0).0, Rect::new(70.0, 0.0, 30.0, 100.0));
    }

    #[test]
    fn asking_for_more_than_there_is_takes_what_is_left() {
        let rect = Rect::new(0.0, 0.0, 50.0, 40.0);
        let (taken, rest) = rect.split_top(999.0);

        assert_eq!(taken.height, 40.0);
        assert_eq!(rest.height, 0.0, "never a negative remainder");
    }

    #[test]
    fn an_inset_never_turns_inside_out() {
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let squeezed = rect.inset(50.0);

        assert_eq!(squeezed.width, 0.0);
        assert_eq!(squeezed.height, 0.0);
    }

    #[test]
    fn overlap_is_the_shared_area_or_nothing() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        assert_eq!(a.intersection(&b), Some(Rect::new(5.0, 5.0, 5.0, 5.0)));

        let c = Rect::new(10.0, 0.0, 10.0, 10.0);
        assert_eq!(a.intersection(&c), None);
    }

    #[test]
    fn a_rect_owns_its_top_left_but_not_its_bottom_right() {
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(rect.contains(0.0, 0.0));
        assert!(rect.contains(9.99, 9.99));
        assert!(
            !rect.contains(10.0, 5.0),
            "the right edge belongs to the next"
        );
    }
}
