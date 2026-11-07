use qrcode::{Color, QrCode};
use std::fmt::Write as _;

use crate::error::Result;

/// Terminal QR code renderer options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QrOptions {
    /// Invert colors (useful for light-theme terminals).
    pub invert: bool,
    /// Quiet zone (border) padding in modules (default: 2).
    pub margin: usize,
}

impl Default for QrOptions {
    fn default() -> Self {
        Self {
            invert: false,
            margin: 2,
        }
    }
}

/// Renders a scannable URL or payload into a compact Unicode terminal string.
///
/// Employs 2-in-1 half-block characters (`▀`, `▄`, `█`, ` `) to compress two
/// vertical QR modules into a single terminal text line.
pub fn render_terminal_qr(data: &str, options: QrOptions) -> Result<String> {
    let code = QrCode::new(data.as_bytes())?;
    let matrix_width = code.width();
    let margin = options.margin;
    let full_width = matrix_width.saturating_add(margin.saturating_mul(2));

    let colors = code.to_colors();

    let get_color = |x: usize, y: usize| -> Color {
        if x < margin || y < margin {
            return Color::Light;
        }
        let mx = x.saturating_sub(margin);
        let my = y.saturating_sub(margin);
        if mx >= matrix_width || my >= matrix_width {
            return Color::Light;
        }
        let index = my.saturating_mul(matrix_width).saturating_add(mx);
        colors.get(index).copied().unwrap_or(Color::Light)
    };

    let full_height = full_width;
    let half_rows = full_height.div_ceil(2);

    let mut output = String::new();

    for row_pair in 0..half_rows {
        let y_top = row_pair.saturating_mul(2);
        let y_bot = y_top.saturating_add(1);

        for x in 0..full_width {
            let top_is_dark = get_color(x, y_top) == Color::Dark;
            let bot_is_dark = if y_bot < full_height {
                get_color(x, y_bot) == Color::Dark
            } else {
                false
            };

            // In typical dark-background terminals:
            // Terminal background is black (dark).
            // Foreground character glyph is white/light.
            // Phone camera requires:
            //   Light (quiet zone) -> Foreground '█'
            //   Dark (QR module)   -> Background ' '
            let (top_active, bot_active) = if options.invert {
                (top_is_dark, bot_is_dark)
            } else {
                (!top_is_dark, !bot_is_dark)
            };

            let ch = match (top_active, bot_active) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };

            let _ = output.write_char(ch);
        }
        output.push('\n');
    }

    Ok(output)
}
