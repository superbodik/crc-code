use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use crc_app::Session;
use crc_theme::{Appearance, Density, Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::{self, CodeMetrics};
use crc_ui::{Offscreen, Shell, ShellState, TextRun};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;

fn main() -> anyhow::Result<()> {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let out = PathBuf::from("site/shots");
    std::fs::create_dir_all(&out)?;

    let mut session = Session::open(&root)?;
    for path in [
        "crates/crc-ui/src/view/logo.rs",
        "crates/crc-editor/src/document.rs",
        "crates/crc-theme/src/palette/dark.rs",
    ] {
        let _ = session.open_file(Path::new(path));
    }
    session.view.branch = "main".to_string();
    session.view.problems = 0;

    let mut canvas = Offscreen::new(WIDTH, HEIGHT)?;
    println!("rendering on {}", canvas.adapter());

    for (name, appearance, density, zen) in [
        ("dark", Appearance::Dark, Density::Balanced, false),
        ("light", Appearance::Light, Density::Balanced, false),
        ("zen", Appearance::Dark, Density::Balanced, true),
        ("dense", Appearance::Dark, Density::Dense, false),
    ] {
        let mut theme = Theme::new(appearance).with_density(density);
        theme.zen = zen;

        let probe = TextRun::new(
            "0000000000",
            Rect::new(0.0, 0.0, 1000.0, 100.0),
            theme.type_scale.code,
            Rgba::hex(0x000000),
        )
        .mono();
        let (width, _) = canvas.measure(&probe);
        let metrics = CodeMetrics {
            char_width: width / 10.0,
            line_height: theme.type_scale.code * crc_theme::typography::LINE_HEIGHT_CODE,
        };

        let layout = Shell::compute(
            Rect::from_size(WIDTH as f32, HEIGHT as f32),
            &theme,
            &ShellState::default(),
        );
        let frame = view::draw(&layout, &theme, &session.view, metrics);
        let pixels = canvas.render_frame(&frame);

        let path = out.join(format!("{name}.png"));
        write_png(&path, WIDTH, HEIGHT, &pixels)?;
        println!("{}", path.display());
    }

    Ok(())
}

fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> anyhow::Result<()> {
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    encoder.write_header()?.write_image_data(pixels)?;
    Ok(())
}
