use crc_theme::typography::{MONO, SANS};
use crc_theme::{Appearance, CONTROL_RING, Density, Highlight, Rgba, Theme, TypeScale};

fn themes() -> [(Appearance, Theme); 2] {
    [
        (Appearance::Light, Theme::light()),
        (Appearance::Dark, Theme::dark()),
    ]
}

#[test]
fn parses_hex_the_way_the_design_states_it() {
    let accent = Rgba::hex(0x7c5cff);
    assert_eq!(
        (accent.r, accent.g, accent.b, accent.a),
        (0x7c, 0x5c, 0xff, 255)
    );
}

#[test]
fn converts_to_linear_for_the_gpu() {
    assert_eq!(Rgba::hex(0x000000).to_linear(), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(Rgba::hex(0xffffff).to_linear(), [1.0, 1.0, 1.0, 1.0]);

    let mid = Rgba::hex(0x808080).to_linear()[0];
    assert!(mid > 0.21 && mid < 0.22, "mid grey linearised to {mid}");
}

#[test]
fn alpha_survives_the_round_trip() {
    let ghost = Rgba::hex(0x7c5cff).with_alpha(128);
    assert_eq!(ghost.a, 128);
    assert!((ghost.to_linear()[3] - 128.0 / 255.0).abs() < 1e-6);
}

#[test]
fn shading_walks_a_colour_towards_black() {
    let base = Rgba::hex(0x808080);

    assert_eq!(base.shade(0.0), base);
    assert_eq!(base.shade(1.0), Rgba::hex(0x000000));
    assert!(base.shade(0.2).relative_luminance() < base.relative_luminance());
    assert_eq!(base.with_alpha(90).shade(0.5).a, 90);
}

#[test]
fn reports_contrast() {
    for (appearance, theme) in themes() {
        let c = theme.chrome;
        println!("--- {appearance:?} ---");
        for (name, fg, bg) in [
            ("text on surface", c.text, c.surface),
            ("text on panel", c.text, c.panel),
            ("muted on surface", c.text_muted, c.surface),
            ("faint on surface", c.text_faint, c.surface),
            ("label on accent", c.text_on_accent, c.accent_solid),
            ("accent on surface", c.accent, c.surface),
            ("keyword", theme.syntax.keyword, c.surface),
            ("function", theme.syntax.function, c.surface),
            ("parameter", theme.syntax.parameter, c.surface),
            ("string", theme.syntax.string, c.surface),
            ("comment", theme.syntax.comment, c.surface),
            ("line number", theme.syntax.line_number, c.surface),
            ("added", theme.diff.added_text, theme.diff.added_background),
            (
                "removed",
                theme.diff.removed_text,
                theme.diff.removed_background,
            ),
        ] {
            println!("{name}: {:.2}", fg.contrast_ratio(bg));
        }
    }
}

#[test]
fn body_text_is_readable() {
    for (appearance, theme) in themes() {
        let c = theme.chrome;
        for (name, background) in [
            ("surface", c.surface),
            ("panel", c.panel),
            ("raised", c.raised),
            ("hover", c.hover),
            ("selected", c.selected),
        ] {
            let ratio = c.text.contrast_ratio(background);
            assert!(
                ratio >= 4.5,
                "{appearance:?}: body text on {name} is only {ratio:.2}:1"
            );
        }
    }
}

#[test]
fn code_is_readable_in_every_syntax_role() {
    for (appearance, theme) in themes() {
        let surface = theme.chrome.surface;
        for highlight in [
            Highlight::Text,
            Highlight::Keyword,
            Highlight::Function,
            Highlight::Parameter,
            Highlight::String,
            Highlight::Number,
            Highlight::Comment,
            Highlight::Punctuation,
        ] {
            let ratio = theme.syntax.color(highlight).contrast_ratio(surface);
            assert!(
                ratio >= 4.5,
                "{appearance:?}: {highlight:?} on the buffer is {ratio:.2}:1"
            );
        }
    }
}

#[test]
fn gutter_numbers_stay_quiet_but_legible() {
    for (appearance, theme) in themes() {
        let surface = theme.chrome.surface;

        let idle = theme.syntax.line_number.contrast_ratio(surface);
        assert!(
            idle >= 3.0,
            "{appearance:?}: gutter numbers are only {idle:.2}:1"
        );

        let active = theme.syntax.line_number_active.contrast_ratio(surface);
        assert!(
            active > idle * 2.0,
            "{appearance:?}: the active number barely stands out"
        );
    }
}

#[test]
fn the_current_line_is_visible_but_does_not_shout() {
    for (appearance, theme) in themes() {
        let ratio = theme
            .syntax
            .current_line
            .contrast_ratio(theme.chrome.surface);
        assert!(
            ratio > 1.02 && ratio < 1.6,
            "{appearance:?}: the current-line band is {ratio:.2}:1 against the buffer"
        );
    }
}

#[test]
fn button_labels_are_readable_on_a_filled_button() {
    for (appearance, theme) in themes() {
        let c = theme.chrome;
        let ratio = c.text_on_accent.contrast_ratio(c.accent_solid);
        assert!(
            ratio >= 4.5,
            "{appearance:?}: a button label is {ratio:.2}:1"
        );
    }
}

#[test]
fn accent_marks_clear_the_non_text_bar() {
    for (appearance, theme) in themes() {
        let c = theme.chrome;
        let ratio = c.accent.contrast_ratio(c.surface);
        assert!(ratio >= 3.0, "{appearance:?}: the caret is {ratio:.2}:1");
    }
}

#[test]
fn diff_text_is_readable_on_its_own_wash() {
    for (appearance, theme) in themes() {
        assert!(
            theme
                .diff
                .added_text
                .contrast_ratio(theme.diff.added_background)
                >= 4.5,
            "{appearance:?}: added text"
        );
        assert!(
            theme
                .diff
                .removed_text
                .contrast_ratio(theme.diff.removed_background)
                >= 4.5,
            "{appearance:?}: removed text"
        );
    }
}

#[test]
fn the_diff_washes_are_tinted_not_flat() {
    for (appearance, theme) in themes() {
        let surface = theme.chrome.surface;
        assert_ne!(
            theme.diff.added_background, surface,
            "{appearance:?}: added lines are not tinted"
        );
        assert_ne!(
            theme.diff.removed_background, surface,
            "{appearance:?}: removed lines are not tinted"
        );
        assert_ne!(theme.diff.added_background, theme.diff.removed_background);
    }
}

#[test]
fn secondary_text_clears_the_lower_bar() {
    for (appearance, theme) in themes() {
        let c = theme.chrome;
        assert!(
            c.text_muted.contrast_ratio(c.surface) >= 3.0,
            "{appearance:?}: muted text"
        );
        assert!(
            c.text_faint.contrast_ratio(c.surface) >= 2.0,
            "{appearance:?}: faint text"
        );
    }
}

#[test]
fn the_dark_theme_is_actually_dark_and_the_light_one_light() {
    let light = Theme::light();
    let dark = Theme::dark();

    assert!(light.chrome.surface.relative_luminance() > 0.7);
    assert!(dark.chrome.surface.relative_luminance() < 0.05);
    assert!(light.chrome.text.relative_luminance() < 0.2);
    assert!(dark.chrome.text.relative_luminance() > 0.6);
}

#[test]
fn no_two_surfaces_share_a_colour() {
    for (appearance, theme) in themes() {
        let c = theme.chrome;
        let surfaces = [
            ("backdrop", c.backdrop),
            ("surface", c.surface),
            ("panel", c.panel),
            ("raised", c.raised),
            ("hover", c.hover),
            ("selected", c.selected),
        ];

        for (index, (name, colour)) in surfaces.iter().enumerate() {
            for (other, other_colour) in surfaces.iter().skip(index + 1) {
                assert_ne!(
                    colour, other_colour,
                    "{appearance:?}: {name} and {other} are the same colour"
                );
            }
        }
    }
}

#[test]
fn selection_is_a_stronger_step_than_hover() {
    for (appearance, theme) in themes() {
        let c = theme.chrome;
        let panel = c.panel.relative_luminance();
        let hover = c.hover.relative_luminance() - panel;
        let selected = c.selected.relative_luminance() - panel;

        assert!(hover != 0.0, "{appearance:?}: hover does not move at all");
        assert_eq!(
            hover.signum(),
            selected.signum(),
            "{appearance:?}: hover and selection step in opposite directions"
        );
        assert!(
            selected.abs() > hover.abs(),
            "{appearance:?}: selection reads weaker than hover"
        );
    }
}

#[test]
fn switching_appearance_keeps_the_layout_settings() {
    let mut theme = Theme::light().with_density(Density::Dense);
    theme.zen = true;
    theme.type_scale = TypeScale::default_scale().scaled(1.25);

    let dark = theme.with_appearance(Appearance::Dark);

    assert_eq!(dark.appearance, Appearance::Dark);
    assert_eq!(dark.density, Density::Dense);
    assert!(dark.zen);
    assert_eq!(dark.type_scale, theme.type_scale);
    assert_eq!(dark.chrome, Theme::dark().chrome);
}

#[test]
fn appearance_flips_between_the_two() {
    assert_eq!(Appearance::Light.flipped(), Appearance::Dark);
    assert_eq!(Appearance::Dark.flipped(), Appearance::Light);
    assert_eq!(Appearance::default(), Appearance::Light);
}

#[test]
fn the_traffic_lights_are_three_distinct_colours() {
    for (appearance, theme) in themes() {
        let c = theme.chrome;
        assert_ne!(c.control_close, c.control_minimize, "{appearance:?}");
        assert_ne!(c.control_minimize, c.control_maximize, "{appearance:?}");
        assert_ne!(c.control_close, c.control_maximize, "{appearance:?}");

        for control in [c.control_close, c.control_minimize, c.control_maximize] {
            assert!(
                control.shade(CONTROL_RING).contrast_ratio(c.panel) >= 2.0,
                "{appearance:?}: a window control has no readable edge on the title bar"
            );
        }
    }
}

#[test]
fn density_tightens_in_one_direction() {
    let calm = Density::Calm.metrics();
    let balanced = Density::Balanced.metrics();
    let dense = Density::Dense.metrics();

    assert!(calm.row_height > balanced.row_height);
    assert!(balanced.row_height > dense.row_height);
    assert!(calm.panel_padding > balanced.panel_padding);
    assert!(balanced.panel_padding > dense.panel_padding);
    assert!(calm.sidebar_width > dense.sidebar_width);
}

#[test]
fn the_calm_profile_hides_the_noisy_parts() {
    let calm = Density::Calm.affordances();
    assert!(!calm.minimap);
    assert!(!calm.bottom_panel);
    assert!(!calm.inline_diagnostics);

    let dense = Density::Dense.affordances();
    assert!(dense.minimap);
    assert!(dense.bottom_panel);
    assert!(dense.inline_diagnostics);
}

#[test]
fn zen_clears_the_screen_whatever_the_density() {
    for density in [Density::Calm, Density::Balanced, Density::Dense] {
        let mut theme = Theme::light().with_density(density);
        theme.zen = true;

        let visible = theme.affordances();
        assert!(!visible.minimap, "{density:?} kept the minimap in zen");
        assert!(!visible.bottom_panel, "{density:?} kept the panel in zen");
        assert!(!visible.breadcrumbs);
        assert!(!visible.inline_diagnostics);
    }
}

#[test]
fn the_font_size_setting_scales_the_whole_scale() {
    let theme = Theme::light().with_code_size(16.0);
    let base = TypeScale::default_scale();
    let factor = 16.0 / base.code;

    assert!((theme.type_scale.code - 16.0).abs() < 1e-4);
    assert!((theme.type_scale.body - base.body * factor).abs() < 1e-4);
    assert!((theme.type_scale.small - base.small * factor).abs() < 1e-4);
}

#[test]
fn the_default_size_leaves_the_scale_alone() {
    let theme = Theme::light().with_code_size(13.0);
    assert!((theme.type_scale.body - 12.0).abs() < 1e-4);
    assert!((theme.type_scale.display - 32.0).abs() < 1e-4);
}

#[test]
fn every_font_stack_ends_in_a_generic_family() {
    for family in [SANS, MONO] {
        let last = family.fallbacks.last().expect("a fallback");
        assert!(
            matches!(*last, "sans-serif" | "serif" | "monospace"),
            "{} falls back to {last}, which may not exist",
            family.primary
        );
    }
}
