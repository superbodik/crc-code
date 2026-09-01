use crc_theme::typography::{MONO, SANS};
use crc_theme::{Density, Highlight, Rgba, Theme, TypeScale};

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
fn reports_contrast() {
    let theme = Theme::light();
    let c = theme.chrome;
    for (name, fg, bg) in [
        ("text on surface", c.text, c.surface),
        ("text on panel", c.text, c.panel),
        ("strong on surface", c.text_strong, c.surface),
        ("muted on surface", c.text_muted, c.surface),
        ("faint on surface", c.text_faint, c.surface),
        ("white on accent", c.text_on_accent, c.accent),
        ("accent on surface", c.accent, c.surface),
        ("border on surface", c.border, c.surface),
        ("keyword on surface", theme.syntax.keyword, c.surface),
        ("function on surface", theme.syntax.function, c.surface),
        ("parameter on surface", theme.syntax.parameter, c.surface),
        ("string on surface", theme.syntax.string, c.surface),
        ("comment on surface", theme.syntax.comment, c.surface),
        (
            "line number on surface",
            theme.syntax.line_number,
            c.surface,
        ),
        (
            "added text on added bg",
            theme.diff.added_text,
            theme.diff.added_background,
        ),
        (
            "removed text on removed bg",
            theme.diff.removed_text,
            theme.diff.removed_background,
        ),
    ] {
        println!("{name}: {:.2}", fg.contrast_ratio(bg));
    }
}

#[test]
fn body_text_is_readable() {
    let c = Theme::light().chrome;
    for (name, background) in [
        ("surface", c.surface),
        ("panel", c.panel),
        ("raised", c.raised),
        ("hover", c.hover),
    ] {
        let ratio = c.text.contrast_ratio(background);
        assert!(ratio >= 4.5, "body text on {name} is only {ratio:.2}:1");
    }
}

#[test]
fn code_is_readable_in_every_syntax_role() {
    let theme = Theme::light();
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
        assert!(ratio >= 4.5, "{highlight:?} on the buffer is {ratio:.2}:1");
    }
}

#[test]
fn gutter_numbers_stay_quiet_but_legible() {
    let theme = Theme::light();
    let surface = theme.chrome.surface;

    let idle = theme.syntax.line_number.contrast_ratio(surface);
    assert!(idle >= 3.0, "gutter numbers are only {idle:.2}:1");

    let active = theme.syntax.line_number_active.contrast_ratio(surface);
    assert!(active > idle * 2.0, "the active number barely stands out");
}

#[test]
fn button_labels_are_readable_on_a_filled_button() {
    let c = Theme::light().chrome;
    let ratio = c.text_on_accent.contrast_ratio(c.accent_solid);
    assert!(ratio >= 4.5, "white on a filled button is {ratio:.2}:1");
}

#[test]
fn accent_marks_clear_the_non_text_bar() {
    let c = Theme::light().chrome;
    assert!(c.accent.contrast_ratio(c.surface) >= 3.0);
}

#[test]
fn diff_text_is_readable_on_its_own_wash() {
    let theme = Theme::light();
    assert!(
        theme
            .diff
            .added_text
            .contrast_ratio(theme.diff.added_background)
            >= 4.5
    );
    assert!(
        theme
            .diff
            .removed_text
            .contrast_ratio(theme.diff.removed_background)
            >= 4.5
    );
}

#[test]
fn secondary_text_clears_the_lower_bar() {
    let c = Theme::light().chrome;
    assert!(c.text_muted.contrast_ratio(c.surface) >= 3.0);
    assert!(c.text_faint.contrast_ratio(c.surface) >= 2.0);
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
