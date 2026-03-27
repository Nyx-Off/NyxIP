use std::sync::Arc;
use tokio::sync::{mpsc, Notify};
use crate::scanner::types::ScanResult;
use crate::scanner::Scanner;
use crate::network;
use crate::oui::OuiDatabase;
use crate::ui::results_table::{SortState, SortColumn};

pub struct NyxApp {
    pub ip_range_input: String,
    pub results: Vec<ScanResult>,
    pub is_scanning: bool,
    pub scan_ports_enabled: bool,
    pub timeout_ms: u32,
    pub scanned_hosts: usize,
    pub total_hosts: usize,
    pub sort_state: SortState,
    pub show_dead: bool,
    pub scan_start_time: Option<std::time::Instant>,
    pub error_message: Option<String>,
    pub show_credits: bool,
    pub show_easter_egg: bool,
    pub konami_buffer: Vec<egui::Key>,

    // Async channels
    result_rx: Option<mpsc::UnboundedReceiver<ScanResult>>,
    progress_rx: Option<mpsc::UnboundedReceiver<(usize, usize)>>,
    cancel_notify: Option<Arc<Notify>>,
    runtime: tokio::runtime::Runtime,
    oui_db: Arc<OuiDatabase>,
}

impl NyxApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let default_range = network::interface::suggest_range();
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        Self {
            ip_range_input: default_range,
            results: Vec::new(),
            is_scanning: false,
            scan_ports_enabled: false,
            timeout_ms: 1000,
            scanned_hosts: 0,
            total_hosts: 0,
            sort_state: SortState::default(),
            show_dead: false,
            scan_start_time: None,
            error_message: None,
            show_credits: false,
            show_easter_egg: false,
            konami_buffer: Vec::new(),
            result_rx: None,
            progress_rx: None,
            cancel_notify: None,
            runtime,
            oui_db: Arc::new(OuiDatabase::new()),
        }
    }

    pub fn start_scan(&mut self) {
        if self.is_scanning {
            return;
        }

        self.error_message = None;

        let ips = match network::range::parse_range(&self.ip_range_input) {
            Ok(ips) => ips,
            Err(e) => {
                self.error_message = Some(format!("Erreur: {}", e));
                return;
            }
        };

        self.results.clear();
        self.is_scanning = true;
        self.scanned_hosts = 0;
        self.total_hosts = ips.len();
        self.scan_start_time = Some(std::time::Instant::now());

        let (result_tx, result_rx) = mpsc::unbounded_channel();
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();
        let cancel = Arc::new(Notify::new());

        self.result_rx = Some(result_rx);
        self.progress_rx = Some(progress_rx);
        self.cancel_notify = Some(cancel.clone());

        let scanner = Scanner::new(self.timeout_ms, self.scan_ports_enabled);
        let oui_db = self.oui_db.clone();

        self.runtime.spawn(async move {
            scanner.scan_range(ips, result_tx, progress_tx, cancel, oui_db).await;
        });
    }

    pub fn stop_scan(&mut self) {
        if let Some(cancel) = &self.cancel_notify {
            cancel.notify_waiters();
        }
        self.is_scanning = false;
    }

    pub fn sort_results(&mut self) {
        let asc = self.sort_state.ascending;
        self.results.sort_by(|a, b| {
            let ord = match self.sort_state.column {
                SortColumn::Ip => u32::from(a.ip).cmp(&u32::from(b.ip)),
                SortColumn::Hostname => a.hostname.cmp(&b.hostname),
                SortColumn::Mac => a.mac.cmp(&b.mac),
                SortColumn::Vendor => a.vendor.cmp(&b.vendor),
                SortColumn::Latency => a.latency_ms.partial_cmp(&b.latency_ms).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Ports => a.open_ports.len().cmp(&b.open_ports.len()),
                SortColumn::Status => format!("{:?}", a.status).cmp(&format!("{:?}", b.status)),
            };
            if asc { ord } else { ord.reverse() }
        });
    }

    pub fn export_csv(&mut self) {
        use rust_xlsxwriter::*;

        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.set_name("NyxIP Scan").ok();

        // Title format
        let title_fmt = Format::new()
            .set_font_size(16)
            .set_bold()
            .set_font_color(Color::RGB(0x00D2FF));

        // Header format - dark background, cyan text, bold
        let header_fmt = Format::new()
            .set_bold()
            .set_font_color(Color::White)
            .set_background_color(Color::RGB(0x1A1A2E))
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0x00D2FF))
            .set_font_size(11);

        // Data format
        let data_fmt = Format::new()
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0x333355))
            .set_font_size(10);

        // Alive format - green text
        let alive_fmt = Format::new()
            .set_font_color(Color::RGB(0x00E676))
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0x333355))
            .set_bold()
            .set_font_size(10);

        // Dead format - red text
        let dead_fmt = Format::new()
            .set_font_color(Color::RGB(0xFF5252))
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0x333355))
            .set_bold()
            .set_font_size(10);

        // MAC format - purple text
        let mac_fmt = Format::new()
            .set_font_color(Color::RGB(0x9664FF))
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0x333355))
            .set_font_size(10);

        // Port format - orange text
        let port_fmt = Format::new()
            .set_font_color(Color::RGB(0xFFAB40))
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0x333355))
            .set_font_size(10);

        // Title row
        let _ = sheet.write_string_with_format(0, 0, "NyxIP Scan Report", &title_fmt);
        let alive = self.results.iter().filter(|r| r.status == crate::scanner::types::HostStatus::Alive).count();
        let _ = sheet.write_string(1, 0, &format!("Date: {} | Total: {} | Alive: {} | Dead: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            self.results.len(), alive, self.results.len() - alive));

        // Column widths
        let _ = sheet.set_column_width(0, 16);  // IP
        let _ = sheet.set_column_width(1, 25);  // Hostname
        let _ = sheet.set_column_width(2, 20);  // MAC
        let _ = sheet.set_column_width(3, 20);  // Vendor
        let _ = sheet.set_column_width(4, 12);  // Latence
        let _ = sheet.set_column_width(5, 25);  // Ports
        let _ = sheet.set_column_width(6, 10);  // Statut

        // Headers at row 3
        let headers = ["IP", "Hostname", "MAC", "Vendor", "Latence (ms)", "Ports", "Statut"];
        for (col, h) in headers.iter().enumerate() {
            let _ = sheet.write_string_with_format(3, col as u16, *h, &header_fmt);
        }

        // Data rows starting at row 4
        for (i, r) in self.results.iter().enumerate() {
            let row = (i + 4) as u32;
            let _ = sheet.write_string_with_format(row, 0, &r.ip.to_string(), &data_fmt);
            let host = if r.hostname.is_empty() { "—" } else { &r.hostname };
            let _ = sheet.write_string_with_format(row, 1, host, &data_fmt);
            let mac = if r.mac.is_empty() { "—" } else { &r.mac };
            let _ = sheet.write_string_with_format(row, 2, mac, &mac_fmt);
            let vendor = if r.vendor.is_empty() { "—" } else { &r.vendor };
            let _ = sheet.write_string_with_format(row, 3, vendor, &data_fmt);
            let lat = r.latency_ms.map(|l| format!("{:.0}", l)).unwrap_or_else(|| "—".to_string());
            let _ = sheet.write_string_with_format(row, 4, &lat, &data_fmt);
            let ports = if r.open_ports.is_empty() { "—".to_string() } else { r.open_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ") };
            let _ = sheet.write_string_with_format(row, 5, &ports, &port_fmt);

            let status_str = r.status.to_string();
            let sfmt = match r.status {
                crate::scanner::types::HostStatus::Alive => &alive_fmt,
                crate::scanner::types::HostStatus::Dead => &dead_fmt,
                _ => &data_fmt,
            };
            let _ = sheet.write_string_with_format(row, 6, &status_str, sfmt);
        }

        // Add autofilter on headers
        let last_row = (self.results.len() + 3) as u32;
        let _ = sheet.autofilter(3, 0, last_row, 6);

        let filename = format!("nyxip_scan_{}.xlsx", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        match workbook.save(&filename) {
            Ok(_) => {
                self.error_message = Some(format!("Export sauvegardé: {}", filename));
            }
            Err(e) => {
                self.error_message = Some(format!("Erreur export: {}", e));
            }
        }
    }

    fn poll_results(&mut self) {
        // Receive scan results
        if let Some(rx) = &mut self.result_rx {
            while let Ok(result) = rx.try_recv() {
                self.results.push(result);
            }
        }

        // Receive progress
        let mut should_sort = false;
        if let Some(rx) = &mut self.progress_rx {
            while let Ok((done, total)) = rx.try_recv() {
                self.scanned_hosts = done;
                self.total_hosts = total;
                if done >= total {
                    self.is_scanning = false;
                    should_sort = true;
                }
            }
        }
        if should_sort {
            self.sort_results();
        }
    }
}

impl eframe::App for NyxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::ui::theme::apply_theme(ctx);

        // Easter egg: Konami code detection
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, .. } = event {
                    self.konami_buffer.push(*key);
                    if self.konami_buffer.len() > 10 {
                        self.konami_buffer.remove(0);
                    }
                    let konami = [
                        egui::Key::ArrowUp, egui::Key::ArrowUp,
                        egui::Key::ArrowDown, egui::Key::ArrowDown,
                        egui::Key::ArrowLeft, egui::Key::ArrowRight,
                        egui::Key::ArrowLeft, egui::Key::ArrowRight,
                        egui::Key::B, egui::Key::A,
                    ];
                    if self.konami_buffer == konami {
                        self.show_easter_egg = true;
                        self.konami_buffer.clear();
                    }
                }
            }
        });

        self.poll_results();

        // Request repaint while scanning
        if self.is_scanning {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // Top panel
        egui::TopBottomPanel::top("scan_panel").show(ctx, |ui| {
            ui.add_space(6.0);
            crate::ui::scan_panel::render(ui, self);
            ui.add_space(6.0);
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let alive_count = self.results.iter()
                    .filter(|r| r.status == crate::scanner::types::HostStatus::Alive)
                    .count();

                ui.label(egui::RichText::new(format!("Hôtes trouvés: {}", alive_count))
                    .color(crate::ui::theme::GREEN));

                ui.separator();

                ui.label(egui::RichText::new(format!("Total scanné: {}", self.results.len()))
                    .color(crate::ui::theme::TEXT_DIM));

                if let Some(start) = self.scan_start_time {
                    ui.separator();
                    let elapsed = start.elapsed();
                    ui.label(egui::RichText::new(format!("Temps: {:.1}s", elapsed.as_secs_f64()))
                        .color(crate::ui::theme::TEXT_DIM));
                }

                // Show/hide dead toggle
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.show_dead,
                        egui::RichText::new("Afficher inactifs").color(crate::ui::theme::TEXT_DIM));
                });
            });
        });

        // Credits window
        egui::Window::new("Crédits")
            .open(&mut self.show_credits)
            .collapsible(false)
            .resizable(false)
            .min_width(320.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("⚡ NyxIP").size(32.0).color(crate::ui::theme::ACCENT_CYAN).strong());
                    ui.label(egui::RichText::new("Network Scanner").size(13.0).color(crate::ui::theme::TEXT_DIM));
                    ui.add_space(6.0);
                });

                ui.separator();
                ui.add_space(6.0);

                egui::Grid::new("credits_grid")
                    .num_columns(2)
                    .spacing([16.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Auteur").color(crate::ui::theme::TEXT_DIM));
                        ui.label(egui::RichText::new("Samy Bensalem").color(crate::ui::theme::TEXT_PRIMARY).strong().size(15.0));
                        ui.end_row();

                        ui.label(egui::RichText::new("Site web").color(crate::ui::theme::TEXT_DIM));
                        ui.hyperlink_to(
                            egui::RichText::new("bensalem.dev").color(crate::ui::theme::ACCENT_CYAN).size(14.0),
                            "https://bensalem.dev"
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Version").color(crate::ui::theme::TEXT_DIM));
                        ui.label(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).color(crate::ui::theme::ACCENT_PURPLE).size(14.0));
                        ui.end_row();

                        ui.label(egui::RichText::new("GitHub").color(crate::ui::theme::TEXT_DIM));
                        ui.hyperlink_to(
                            egui::RichText::new("github.com/Nyx-Off/NyxIP").color(crate::ui::theme::ACCENT_CYAN).size(13.0),
                            "https://github.com/Nyx-Off/NyxIP"
                        );
                        ui.end_row();
                    });

                ui.add_space(4.0);
            });

        // Easter egg window
        if self.show_easter_egg {
            egui::Window::new("🎮 !!!")
                .open(&mut self.show_easter_egg)
                .collapsible(false)
                .resizable(true)
                .default_pos(egui::pos2(350.0, 200.0))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(15.0);
                        ui.label(egui::RichText::new("🎉").size(50.0));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Konami Code Activé !").size(22.0).color(crate::ui::theme::ACCENT_CYAN).strong());
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Tu as trouvé l'easter egg !").size(15.0).color(crate::ui::theme::TEXT_PRIMARY));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("↑ ↑ ↓ ↓ ← → ← → B A").size(13.0).color(crate::ui::theme::ACCENT_PURPLE).monospace());
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("NyxIP — Forgé dans l'ombre du réseau").size(12.0).color(crate::ui::theme::TEXT_DIM).italics());
                        ui.add_space(10.0);
                    });
                });
        }

        // Error message
        if let Some(err) = &self.error_message {
            egui::TopBottomPanel::top("error").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("⚠ {}", err))
                        .color(crate::ui::theme::RED));
                });
            });
        }

        // Central panel - results table
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.results.is_empty() && !self.is_scanning {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("Entrez une plage IP et lancez le scan")
                        .size(18.0)
                        .color(crate::ui::theme::TEXT_DIM));
                });
            } else {
                crate::ui::results_table::render(ui, self);
            }
        });
    }
}
