use egui::{Ui, RichText, Color32, Vec2};
use crate::app::NyxApp;
use crate::ui::theme::*;

pub fn render(ui: &mut Ui, app: &mut NyxApp) {
    // Row 1: Main controls
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;

        ui.label(RichText::new("⚡ NyxIP").size(22.0).color(ACCENT_CYAN).strong());
        ui.separator();

        ui.label(RichText::new("Plage IP:").color(TEXT_PRIMARY).size(14.0));
        let response = ui.add_sized(
            Vec2::new(300.0, 30.0),
            egui::TextEdit::singleline(&mut app.ip_range_input)
                .hint_text("ex: 192.168.1.0/24")
                .font(egui::TextStyle::Monospace),
        );

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            app.start_scan();
        }

        if app.is_scanning {
            if ui.add_sized(
                Vec2::new(90.0, 30.0),
                egui::Button::new(RichText::new("⏹ Stop").color(Color32::WHITE).strong().size(14.0))
                    .fill(RED)
            ).on_hover_text("Arrêter le scan").clicked() {
                app.stop_scan();
            }
        } else {
            if ui.add_sized(
                Vec2::new(90.0, 30.0),
                egui::Button::new(RichText::new("▶ Scan").color(BG_DARK).strong().size(14.0))
                    .fill(ACCENT_CYAN)
            ).on_hover_text("Lancer le scan").clicked() {
                app.start_scan();
            }
        }
    });

    ui.add_space(2.0);

    // Row 2: Options
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;

        ui.add_space(4.0);
        ui.checkbox(&mut app.scan_ports_enabled, RichText::new("Scanner les ports").color(TEXT_PRIMARY));

        ui.separator();

        ui.label(RichText::new("Timeout:").color(TEXT_DIM));
        ui.add(egui::DragValue::new(&mut app.timeout_ms)
            .range(100..=10000)
            .suffix(" ms")
            .speed(50));

        ui.separator();

        if !app.results.is_empty() {
            if ui.add_sized(
                Vec2::new(100.0, 24.0),
                egui::Button::new(RichText::new("📋 Exporter").color(TEXT_PRIMARY))
            ).clicked() {
                app.export();
            }
        }

        // Credits button
        if ui.add_sized(
            Vec2::new(28.0, 24.0),
            egui::Button::new(RichText::new("ℹ").size(16.0).color(TEXT_PRIMARY))
        ).on_hover_text("Crédits").clicked() {
            app.show_credits = !app.show_credits;
        }
    });

    // Progress bar
    if app.is_scanning {
        ui.add_space(4.0);
        let progress = if app.total_hosts > 0 {
            app.scanned_hosts as f32 / app.total_hosts as f32
        } else {
            0.0
        };
        let text = format!("{}/{} hosts scannés ({:.0}%)", app.scanned_hosts, app.total_hosts, progress * 100.0);

        let desired_size = egui::vec2(ui.available_width(), 22.0);
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            // Background
            painter.rect_filled(rect, 4.0, BG_WIDGET);
            // Filled portion
            let filled_width = rect.width() * progress;
            let mut filled_rect = rect;
            filled_rect.set_width(filled_width);
            painter.rect_filled(filled_rect, 4.0, ACCENT_CYAN);

            // Draw text twice with clipping for dual-color effect:
            // White text on the unfilled (dark) part, dark text on the filled (cyan) part
            let font = egui::FontId::proportional(12.0);
            let text_pos = rect.center();

            // First: white text everywhere (will be behind the dark text on filled area)
            painter.text(text_pos, egui::Align2::CENTER_CENTER, &text, font.clone(), Color32::WHITE);

            // Second: dark text clipped to the filled area only
            let clip_rect = filled_rect;
            painter.with_clip_rect(clip_rect).text(
                text_pos, egui::Align2::CENTER_CENTER, &text, font, BG_DARK,
            );
        }
    }
}
