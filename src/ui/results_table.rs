use egui::{Ui, RichText, ScrollArea};
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

fn column_header(ui: &mut Ui, app: &mut NyxApp, name: &str, col: SortColumn) {
    let is_sorted = app.sort_state.column == col;
    let arrow = if is_sorted {
        if app.sort_state.ascending { " ▲" } else { " ▼" }
    } else { "" };

    let text = format!("{}{}", name, arrow);
    let label = RichText::new(text).color(ACCENT_CYAN).strong().size(13.0);

    if ui.add(egui::Label::new(label).sense(egui::Sense::click())).clicked() {
        if app.sort_state.column == col {
            app.sort_state.ascending = !app.sort_state.ascending;
        } else {
            app.sort_state.column = col;
            app.sort_state.ascending = true;
        }
        app.sort_results();
    }
}

pub fn render(ui: &mut Ui, app: &mut NyxApp) {
    // Use Grid for proper column alignment
    let grid_id = ui.id().with("results_grid");

    // Header row
    egui::Grid::new(grid_id.with("header"))
        .num_columns(7)
        .spacing([12.0, 4.0])
        .min_col_width(60.0)
        .striped(false)
        .show(ui, |ui| {
            column_header(ui, app, "IP", SortColumn::Ip);
            column_header(ui, app, "Hostname", SortColumn::Hostname);
            column_header(ui, app, "MAC", SortColumn::Mac);
            column_header(ui, app, "Vendor", SortColumn::Vendor);
            column_header(ui, app, "Latence", SortColumn::Latency);
            column_header(ui, app, "Ports", SortColumn::Ports);
            column_header(ui, app, "Statut", SortColumn::Status);
            ui.end_row();
        });

    ui.separator();

    // Data rows in scrollable area
    ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        egui::Grid::new(grid_id.with("data"))
            .num_columns(7)
            .spacing([12.0, 2.0])
            .min_col_width(60.0)
            .striped(true)
            .show(ui, |ui| {
                for result in &app.results {
                    if !app.show_dead && result.status == HostStatus::Dead {
                        continue;
                    }

                    let status_color = match result.status {
                        HostStatus::Alive => GREEN,
                        HostStatus::Dead => RED,
                        HostStatus::Unknown => ORANGE,
                    };

                    let ip_str = result.ip.to_string();

                    // IP - with context menu
                    let ip_resp = ui.add(egui::Label::new(
                        RichText::new(&ip_str).monospace().color(TEXT_PRIMARY)
                    ).sense(egui::Sense::click()));

                    ip_resp.context_menu(|ui| {
                        ui.label(RichText::new(format!("📍 {}", ip_str)).strong().color(ACCENT_CYAN));
                        ui.separator();

                        // Open in browser submenu
                        ui.menu_button(RichText::new("🌐 Ouvrir dans le navigateur").color(TEXT_PRIMARY), |ui| {
                            if ui.button("HTTP (port 80)").clicked() {
                                let _ = open::that(format!("http://{}", ip_str));
                                ui.close_menu();
                            }
                            if ui.button("HTTPS (port 443)").clicked() {
                                let _ = open::that(format!("https://{}", ip_str));
                                ui.close_menu();
                            }
                            if ui.button("Port 8080").clicked() {
                                let _ = open::that(format!("http://{}:8080", ip_str));
                                ui.close_menu();
                            }
                            if ui.button("Port 8443").clicked() {
                                let _ = open::that(format!("https://{}:8443", ip_str));
                                ui.close_menu();
                            }
                            if ui.button("Port 3000").clicked() {
                                let _ = open::that(format!("http://{}:3000", ip_str));
                                ui.close_menu();
                            }
                            if ui.button("Port 9090").clicked() {
                                let _ = open::that(format!("http://{}:9090", ip_str));
                                ui.close_menu();
                            }
                        });

                        if ui.button(RichText::new("📁 Ouvrir dans l'explorateur (SMB)").color(TEXT_PRIMARY)).clicked() {
                            let _ = open::that(format!("\\\\{}", ip_str));
                            ui.close_menu();
                        }

                        if ui.button(RichText::new("🖥️ Connexion Bureau à distance (RDP)").color(TEXT_PRIMARY)).clicked() {
                            let _ = std::process::Command::new("mstsc")
                                .arg(format!("/v:{}", ip_str))
                                .spawn();
                            ui.close_menu();
                        }

                        if ui.button(RichText::new("💻 Ouvrir terminal SSH").color(TEXT_PRIMARY)).clicked() {
                            let _ = std::process::Command::new("cmd")
                                .args(["/c", "start", "ssh", &ip_str])
                                .spawn();
                            ui.close_menu();
                        }

                        ui.separator();

                        if ui.button(RichText::new("📋 Copier l'IP").color(TEXT_PRIMARY)).clicked() {
                            ui.ctx().copy_text(ip_str.clone());
                            ui.close_menu();
                        }

                        if !result.mac.is_empty() {
                            let mac_copy = result.mac.clone();
                            if ui.button(RichText::new("📋 Copier le MAC").color(TEXT_PRIMARY)).clicked() {
                                ui.ctx().copy_text(mac_copy);
                                ui.close_menu();
                            }
                        }
                    });

                    // Hostname
                    let hostname_text = if result.hostname.is_empty() { "—" } else { &result.hostname };
                    ui.label(RichText::new(hostname_text).color(
                        if result.hostname.is_empty() { TEXT_DIM } else { TEXT_PRIMARY }
                    ));

                    // MAC
                    let mac_text = if result.mac.is_empty() { "—" } else { &result.mac };
                    ui.label(RichText::new(mac_text).monospace().color(
                        if result.mac.is_empty() { TEXT_DIM } else { ACCENT_PURPLE }
                    ));

                    // Vendor
                    let vendor_text = if result.vendor.is_empty() { "—" } else { &result.vendor };
                    ui.label(RichText::new(vendor_text).color(TEXT_DIM));

                    // Latency
                    let latency_text = match result.latency_ms {
                        Some(ms) => format!("{:.0} ms", ms),
                        None => "—".to_string(),
                    };
                    ui.label(RichText::new(&latency_text).color(
                        if result.latency_ms.is_some() { GREEN } else { TEXT_DIM }
                    ));

                    // Ports
                    let ports_text = if result.open_ports.is_empty() {
                        "—".to_string()
                    } else {
                        result.open_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")
                    };
                    ui.label(RichText::new(&ports_text).monospace().color(
                        if result.open_ports.is_empty() { TEXT_DIM } else { ORANGE }
                    ));

                    // Status
                    ui.label(RichText::new(result.status.to_string()).color(status_color).strong());

                    ui.end_row();
                }
            });
    });
}
