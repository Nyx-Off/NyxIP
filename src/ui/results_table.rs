use egui::{Ui, RichText, ScrollArea, Color32};
use crate::app::NyxApp;
use crate::scanner::types::HostStatus;
use crate::ui::theme::*;

#[derive(Clone, Copy, PartialEq)]
pub enum SortColumn {
    Ip,
    Hostname,
    Mac,
    Vendor,
    Latency,
    Ports,
    Status,
}

#[derive(Clone, Copy)]
pub struct SortState {
    pub column: SortColumn,
    pub ascending: bool,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            column: SortColumn::Ip,
            ascending: true,
        }
    }
}


pub fn render(ui: &mut Ui, app: &mut NyxApp) {
    let col_widths: [f32; 7] = [130.0, 170.0, 155.0, 140.0, 80.0, 150.0, 70.0];
    let headers = [
        ("IP", SortColumn::Ip),
        ("Hostname", SortColumn::Hostname),
        ("MAC", SortColumn::Mac),
        ("Vendor", SortColumn::Vendor),
        ("Latence", SortColumn::Latency),
        ("Ports", SortColumn::Ports),
        ("Statut", SortColumn::Status),
    ];

    // Header row - manual layout matching data columns
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        ui.add_space(4.0);
        for (i, (name, col)) in headers.iter().enumerate() {
            let is_sorted = app.sort_state.column == *col;
            let arrow = if is_sorted {
                if app.sort_state.ascending { " ^" } else { " v" }
            } else { "" };
            let text = format!("{}{}", name, arrow);
            let label = RichText::new(text).color(ACCENT_CYAN).strong().size(13.0);
            let resp = ui.add_sized(
                egui::vec2(col_widths[i], 20.0),
                egui::Label::new(label).sense(egui::Sense::click()),
            );
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                if app.sort_state.column == *col {
                    app.sort_state.ascending = !app.sort_state.ascending;
                } else {
                    app.sort_state.column = *col;
                    app.sort_state.ascending = true;
                }
                app.sort_results();
            }
        }
    });

    ui.separator();

    // Data rows in scrollable area
    let row_height = 22.0;
    let col_widths: [f32; 7] = [130.0, 170.0, 155.0, 140.0, 80.0, 150.0, 70.0];

    ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        for (row_idx, result) in app.results.iter().enumerate() {
            if !app.show_dead && result.status == HostStatus::Dead {
                continue;
            }

            let status_color = match result.status {
                HostStatus::Alive => GREEN,
                HostStatus::Dead => RED,
                HostStatus::Unknown => ORANGE,
            };

            let ip_str = result.ip.to_string();
            let _row_id = ui.id().with(("row", row_idx));

            // Allocate full row rect for hover detection
            let total_width: f32 = col_widths.iter().sum::<f32>() + 12.0 * 6.0;
            let available_w = ui.available_width().max(total_width);
            let (row_rect, row_resp) = ui.allocate_exact_size(
                egui::vec2(available_w, row_height),
                egui::Sense::click(),
            );

            // Hover highlight
            if row_resp.hovered() {
                ui.painter().rect_filled(row_rect, 0.0, Color32::from_rgba_premultiplied(0, 210, 255, 18));
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            } else if row_idx % 2 == 1 {
                ui.painter().rect_filled(row_rect, 0.0, Color32::from_rgba_premultiplied(255, 255, 255, 5));
            }

            // Context menu on the entire row
            row_resp.context_menu(|ui| {
                ui.label(RichText::new(format!("  {}", ip_str)).strong().color(ACCENT_CYAN));
                ui.separator();

                ui.menu_button(RichText::new("  Ouvrir dans le navigateur").color(TEXT_PRIMARY), |ui| {
                    for (label, url) in [
                        ("HTTP (port 80)", format!("http://{}", ip_str)),
                        ("HTTPS (port 443)", format!("https://{}", ip_str)),
                        ("Port 8080", format!("http://{}:8080", ip_str)),
                        ("Port 8443", format!("https://{}:8443", ip_str)),
                        ("Port 3000", format!("http://{}:3000", ip_str)),
                        ("Port 9090", format!("http://{}:9090", ip_str)),
                    ] {
                        if ui.button(label).clicked() {
                            let _ = open::that(&url);
                            ui.close_menu();
                        }
                    }
                });

                if ui.button(RichText::new("  Ouvrir dans l'explorateur (SMB)").color(TEXT_PRIMARY)).clicked() {
                    let _ = open::that(format!("\\\\{}", ip_str));
                    ui.close_menu();
                }

                if ui.button(RichText::new("  Bureau a distance (RDP)").color(TEXT_PRIMARY)).clicked() {
                    let _ = std::process::Command::new("mstsc")
                        .arg(format!("/v:{}", ip_str))
                        .spawn();
                    ui.close_menu();
                }

                if ui.button(RichText::new("  Terminal SSH").color(TEXT_PRIMARY)).clicked() {
                    let _ = std::process::Command::new("cmd")
                        .args(["/c", "start", "ssh", &ip_str])
                        .spawn();
                    ui.close_menu();
                }

                ui.separator();

                if ui.button(RichText::new("  Copier l'IP").color(TEXT_PRIMARY)).clicked() {
                    ui.ctx().copy_text(ip_str.clone());
                    ui.close_menu();
                }

                if !result.mac.is_empty() {
                    let mac_copy = result.mac.clone();
                    if ui.button(RichText::new("  Copier le MAC").color(TEXT_PRIMARY)).clicked() {
                        ui.ctx().copy_text(mac_copy);
                        ui.close_menu();
                    }
                }
            });

            // Draw cells on top of the row rect
            let painter = ui.painter();
            let mut x = row_rect.left() + 4.0;
            let cy = row_rect.center().y;
            let font = egui::FontId::proportional(12.0);
            let mono = egui::FontId::monospace(12.0);

            let cells: Vec<(String, &egui::FontId, Color32)> = vec![
                (ip_str, &mono, TEXT_PRIMARY),
                (if result.hostname.is_empty() { "---".into() } else { result.hostname.clone() },
                    &font, if result.hostname.is_empty() { TEXT_DIM } else { TEXT_PRIMARY }),
                (if result.mac.is_empty() { "---".into() } else { result.mac.clone() },
                    &mono, if result.mac.is_empty() { TEXT_DIM } else { ACCENT_PURPLE }),
                (if result.vendor.is_empty() { "---".into() } else { result.vendor.clone() },
                    &font, TEXT_DIM),
                (result.latency_ms.map(|l| format!("{:.0} ms", l)).unwrap_or_else(|| "---".into()),
                    &font, if result.latency_ms.is_some() { GREEN } else { TEXT_DIM }),
                (if result.open_ports.is_empty() { "---".into() } else { result.open_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ") },
                    &mono, if result.open_ports.is_empty() { TEXT_DIM } else { ORANGE }),
                (result.status.to_string(), &font, status_color),
            ];

            for (i, (text, f, color)) in cells.iter().enumerate() {
                painter.text(
                    egui::pos2(x + 2.0, cy),
                    egui::Align2::LEFT_CENTER,
                    text,
                    (*f).clone(),
                    *color,
                );
                x += col_widths[i] + 12.0;
            }
        }
    });
}
