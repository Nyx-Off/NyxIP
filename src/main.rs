#![windows_subsystem = "windows"]

mod app;
mod ui;
mod scanner;
mod network;
mod oui;

fn generate_icon() -> egui::IconData {
    let size = 64usize;
    let mut rgba = vec![0u8; size * size * 4];

    // Fill with black background
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[0] = 0;   // R
        pixel[1] = 0;   // G
        pixel[2] = 0;   // B
        pixel[3] = 255; // A
    }

    // Draw "N" in cyan (0, 210, 255) - grunge style with rough edges
    let cyan = [0u8, 210, 255, 255];

    // N character bitmap: left stroke, diagonal, right stroke
    // Working in a 64x64 canvas, N occupies roughly columns 12-52, rows 8-56
    let x_start = 12;
    let x_end = 52;
    let y_start = 8;
    let y_end = 56;
    let stroke = 8;
    let height = y_end - y_start;

    for y in y_start..y_end {
        for x in x_start..x_end {
            let draw =
                // Left vertical stroke
                (x >= x_start && x < x_start + stroke) ||
                // Right vertical stroke
                (x >= x_end - stroke && x < x_end) ||
                // Diagonal connecting top-left to bottom-right
                {
                    let progress = (y - y_start) as f32 / height as f32;
                    let diag_center = x_start as f32 + stroke as f32 / 2.0 + progress * (x_end as f32 - x_start as f32 - stroke as f32);
                    let dist = (x as f32 - diag_center).abs();
                    dist < (stroke as f32 / 2.0 + 1.0)
                };

            if draw {
                // Add grunge effect: slight random variation using position-based noise
                let noise = ((x * 7 + y * 13) % 5) as i16 - 2;
                let r = (cyan[0] as i16 + noise * 3).clamp(0, 255) as u8;
                let g = (cyan[1] as i16 + noise * 5).clamp(0, 255) as u8;
                let b = (cyan[2] as i16 + noise * 2).clamp(0, 255) as u8;

                // Rough edges: skip some edge pixels for grunge look
                let is_edge = x == x_start || x == x_end - 1 || y == y_start || y == y_end - 1;
                if !is_edge || (x + y) % 3 != 0 {
                    let idx = (y * size + x) * 4;
                    rgba[idx] = r;
                    rgba[idx + 1] = g;
                    rgba[idx + 2] = b;
                    rgba[idx + 3] = 255;
                }
            }
        }
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

fn main() -> eframe::Result<()> {
    let icon = generate_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 650.0])
            .with_min_inner_size([800.0, 400.0])
            .with_title("NyxIP — Network Scanner")
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    eframe::run_native(
        "NyxIP",
        options,
        Box::new(|cc| Ok(Box::new(app::NyxApp::new(cc)))),
    )
}
