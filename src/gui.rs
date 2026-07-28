#![windows_subsystem = "windows"]

mod api_client;
mod file_ops;

use eframe::egui;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use skill_manager_mcp::management::service_ctrl;
use skill_manager_mcp::tools::skill_scanner;

#[cfg(windows)]
fn detect_windows_dark_mode() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    
    let personalize = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    match personalize {
        Ok(key) => {
            let value: u32 = key.get_value("AppsUseLightTheme").unwrap_or(1);
            value == 0 // 0 = 深色, 1 = 浅色
        }
        Err(_) => true,
    }
}

#[cfg(not(windows))]
fn detect_windows_dark_mode() -> bool { true }

#[cfg(windows)]
fn get_hwnd_value(cc: &eframe::CreationContext<'_>) -> Option<isize> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let handle = cc.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get() as isize),
        _ => None,
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SkillEntry {
    name: String,
    description: String,
    tags: String,
    dir_name: String,
    enabled: bool,
}

struct App {
    port: String,
    web_port: String,
    listen_host: String,
    service_running: bool,
    auto_start: bool,
    status_msg: String,
    status_time: f64,

    skills: Vec<SkillEntry>,
    skills_dir: PathBuf,

    show_import_window: bool,
    popup_close_flag: Rc<Cell<bool>>,
    import_log: Rc<RefCell<Vec<String>>>,
    import_pending_paths: Rc<RefCell<Vec<std::path::PathBuf>>>,
    import_refresh_flag: Rc<Cell<bool>>,
    import_name: String,
    import_tags: String,
    import_note: String,
    import_save_as_skill: bool,
    imported_count: usize,
    show_delete_confirm: bool,
    delete_target: String,
    use_tray: bool,
    tray_icon: Option<tray_icon::TrayIcon>,
    tray_open_id: muda::MenuId,
    tray_exit_id: muda::MenuId,
    dark_mode_override: Option<bool>,
    #[cfg(windows)]
    hwnd: Option<isize>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();

        let skills_dir = exe_dir.join("skills");
        std::fs::create_dir_all(&skills_dir).ok();

        let port = file_ops::read_config_port(&exe_dir);
        let web_port = file_ops::read_config_web_port(&exe_dir);
        let listen_host = file_ops::read_config_listen_host(&exe_dir);
        let use_tray = file_ops::read_config_use_tray(&exe_dir);
        let dark_mode_override = file_ops::read_config_dark_mode_override(&exe_dir);

        api_client::init_base_url(&format!("{}:{}", listen_host, web_port));

        let auto_start = file_ops::read_auto_start();

        let mut service_running = service_ctrl::check_service_running();
        if !service_running {
            let _ = service_ctrl::start_service(&exe_dir);
            std::thread::sleep(std::time::Duration::from_millis(800));
            service_running = service_ctrl::check_service_running();
        }

        let skills = Self::scan_local_skills_static(&skills_dir);

        #[cfg(windows)]
        let hwnd = get_hwnd_value(cc);
        #[cfg(not(windows))]
        let hwnd: Option<()> = None;

        let (tray_icon, tray_open_id, tray_exit_id) = if use_tray {
            match create_tray_icon() {
                Ok((icon, open_id, exit_id)) => (Some(icon), open_id, exit_id),
                Err(_) => (
                    None,
                    muda::MenuId(String::new()),
                    muda::MenuId(String::new()),
                ),
            }
        } else {
            (
                None,
                muda::MenuId(String::new()),
                muda::MenuId(String::new()),
            )
        };

        let open_id_for_handler = tray_open_id.clone();
        let exit_id_for_handler = tray_exit_id.clone();
        let hwnd_for_handler = hwnd;
        muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            #[cfg(windows)]
            {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::{
                    SW_RESTORE, SetForegroundWindow, ShowWindow,
                };
                if event.id.0 == open_id_for_handler.0 {
                    if let Some(hwnd_value) = hwnd_for_handler {
                        let hwnd = HWND(hwnd_value as *mut core::ffi::c_void);
                        unsafe {
                            let _ = ShowWindow(hwnd, SW_RESTORE);
                            let _ = SetForegroundWindow(hwnd);
                        }
                    }
                } else if event.id.0 == exit_id_for_handler.0 {
                    std::process::exit(0);
                }
            }
        }));

        Self {
            port,
            web_port,
            listen_host,
            service_running,
            auto_start,
            status_msg: String::new(),
            status_time: 0.0,
            skills,
            skills_dir,
            show_import_window: false,
            popup_close_flag: Rc::new(Cell::new(false)),
            import_log: Rc::new(RefCell::new(Vec::new())),
            import_pending_paths: Rc::new(RefCell::new(Vec::new())),
            import_refresh_flag: Rc::new(Cell::new(false)),
            import_name: String::new(),
            import_tags: String::new(),
            import_note: String::new(),
            import_save_as_skill: false,
            imported_count: 0,
            show_delete_confirm: false,
            delete_target: String::new(),
            use_tray,
            tray_icon,
            tray_open_id,
            tray_exit_id,
            dark_mode_override,
            #[cfg(windows)]
            hwnd,
        }
    }

    fn start_service(&mut self) {
        self.status_msg = "正在启动...".to_string();
        let exe = std::env::current_exe().unwrap();
        let dir = exe.parent().unwrap();
        match service_ctrl::start_service(dir) {
            Ok(msg) => {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                self.service_running = service_ctrl::check_service_running();
                self.status_msg = if self.service_running { msg } else { "启动失败".to_string() };
            }
            Err(e) => {
                self.service_running = false;
                self.status_msg = format!("启动失败: {}", e);
            }
        }
    }

    fn stop_service(&mut self) {
        self.status_msg = "正在停止...".to_string();
        let _ = service_ctrl::stop_service();
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(300));
            if !service_ctrl::check_service_running() {
                break;
            }
        }
        self.service_running = false;
        self.status_msg = "服务已停止".to_string();
    }

    fn set_status(&mut self, msg: &str, ctx: &egui::Context) {
        self.status_msg = msg.to_string();
        self.status_time = ctx.input(|i| i.time);
    }

    fn refresh_status(&mut self, ctx: &egui::Context) {
        self.service_running = service_ctrl::check_service_running();
        let msg = if self.service_running { "服务运行中" } else { "服务未运行" };
        self.set_status(msg, ctx);
    }

    fn browser_host(&self) -> &str {
        if self.listen_host == "0.0.0.0" { "127.0.0.1" } else { &self.listen_host }
    }

    fn open_browser_web_admin(&self) {
        let url = format!("http://{}:{}/web-admin/", self.browser_host(), self.web_port);
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .creation_flags(0x08000000)
            .spawn()
            .ok();
    }

    fn open_browser_about(&self) {
        let url = format!("http://{}:{}", self.browser_host(), self.web_port);
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .creation_flags(0x08000000)
            .spawn()
            .ok();
    }

    fn toggle_auto_start(&self) {
        file_ops::toggle_auto_start(self.auto_start);
    }

    fn save_port_to_config(&self) {
        file_ops::save_port_to_config(&self.port, &self.web_port);
    }

    fn refresh_skills(&mut self) {
        // 先调用API刷新数据库（扫描目录 + 写入数据库）
        let _ = api_client::refresh_skills();
        
        // 再从API读取最新数据
        match api_client::list_skills() {
            Ok(list) => {
                self.skills = list.into_iter().map(|s| SkillEntry {
                    name: s.name,
                    description: s.description,
                    tags: s.tags,
                    dir_name: s.dir_name,
                    enabled: s.enabled,
                }).collect();
            }
            Err(_) => {
                // API不可用时，扫描目录
                self.skills = self.scan_local_skills();
            }
        }
    }

    fn scan_local_skills_static(skills_dir: &std::path::Path) -> Vec<SkillEntry> {
        let mut skills = Vec::new();
        if !skills_dir.exists() {
            return skills;
        }
        if let Ok(read_dir) = std::fs::read_dir(skills_dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let skill_file = path.join("SKILL.md");
                if !skill_file.exists() {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&skill_file) {
                    let (name, description, tags) = skill_scanner::parse_front_matter(&content);
                    let dir_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    skills.push(SkillEntry {
                        name: if name.is_empty() { dir_name.clone() } else { name },
                        description,
                        tags,
                        dir_name,
                        enabled: true,
                    });
                }
            }
        }
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        
        // 从数据库同步状态（enabled + 过滤已删除）
        if let Ok(db_skills) = api_client::list_skills() {
            let db_map: std::collections::HashMap<String, bool> = db_skills
                .into_iter()
                .map(|s| (s.name, s.enabled))
                .collect();
            for skill in &mut skills {
                if let Some(&enabled) = db_map.get(&skill.name) {
                    skill.enabled = enabled;
                }
            }
        }
        
        // 过滤掉数据库中已删除的skill
        if let Ok(all_db_skills) = api_client::list_skills() {
            let db_names: std::collections::HashSet<String> = all_db_skills
                .into_iter()
                .map(|s| s.name)
                .collect();
            skills.retain(|s| db_names.contains(&s.name));
        }
        
        skills
    }

    fn scan_local_skills(&self) -> Vec<SkillEntry> {
        Self::scan_local_skills_static(&self.skills_dir)
    }

    fn delete_skill(&mut self, name: &str) {
        // 调用API删除（API会移动文件到回收站 + 更新数据库）
        let _ = api_client::delete_skill(name);
        self.skills.retain(|s| s.name != name);
    }

    fn apply_dark_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(18, 20, 24);
        style.visuals.panel_fill = egui::Color32::from_rgb(28, 31, 38);
        style.visuals.window_fill = egui::Color32::from_rgb(28, 31, 38);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 39, 48);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 44, 56);
        style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(40, 44, 56);
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200));
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 66, 82));
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(50, 56, 72);
        style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(50, 56, 72);
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220));
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 120, 200));
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 56, 72);
        style.visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(50, 56, 72);
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 240, 240));
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 120, 200));
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(60, 100, 180);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(140, 180, 240));
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(18, 20, 24);
        self.apply_common_style(&mut style);
        ctx.set_style(style);
    }

    fn apply_light_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = false;
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(245, 245, 245);
        style.visuals.panel_fill = egui::Color32::from_rgb(250, 250, 252);
        style.visuals.window_fill = egui::Color32::from_rgb(252, 252, 254);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(235, 235, 238);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(230, 230, 235);
        style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(230, 230, 235);
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 30, 30));
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 185));
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(210, 215, 230);
        style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(210, 215, 230);
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(10, 10, 10));
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 100, 180));
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(200, 210, 230);
        style.visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(200, 210, 230);
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0));
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 100, 180));
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(80, 120, 200);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
        self.apply_common_style(&mut style);
        ctx.set_style(style);
    }

    fn apply_common_style(&self, style: &mut egui::Style) {
        style.visuals.button_frame = true;
        style.visuals.window_corner_radius = 6.0.into();
        style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 160, 165));
        style.text_styles.clear();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(20.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        );
    }
}

// ── UI ──

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.use_tray && self.tray_icon.is_some() {
            let close_requested = ctx.input(|i| i.viewport().close_requested());
            if close_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                #[cfg(windows)]
                if let Some(hwnd_value) = self.hwnd {
                    unsafe {
                        let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                            windows::Win32::Foundation::HWND(hwnd_value as *mut core::ffi::c_void),
                            windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
                        );
                    }
                }
            }
        }

        if self.import_refresh_flag.get() {
            self.import_refresh_flag.set(false);
            self.refresh_skills();
        }

        let is_dark = match self.dark_mode_override {
            Some(v) => v,
            None => detect_windows_dark_mode(),
        };
        if is_dark {
            self.apply_dark_theme(ctx);
        } else {
            self.apply_light_theme(ctx);
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            if !self.status_msg.is_empty() {
                let now = ctx.input(|i| i.time);
                if now - self.status_time > 3.0 {
                    self.status_msg.clear();
                }
            }
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !self.status_msg.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.status_msg)
                                .color(egui::Color32::from_rgb(180, 180, 180))
                                .size(12.0),
                        );
                        ui.separator();
                    }
                    let (color, text) = if self.service_running {
                        (egui::Color32::GREEN, "服务状态：运行中")
                    } else {
                        (egui::Color32::from_rgb(150, 150, 150), "服务状态：已停止")
                    };
                    ui.label(
                        egui::RichText::new(format!("● {}", text))
                            .color(color)
                            .size(13.0),
                    );
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(12.0);

                let (card_bg, card_border) = if is_dark {
                    (egui::Color32::from_rgb(28, 31, 38), egui::Color32::from_rgb(50, 55, 68))
                } else {
                    (egui::Color32::from_rgb(245, 245, 248), egui::Color32::from_rgb(200, 200, 205))
                };
                let card_frame = egui::Frame::NONE
                    .fill(card_bg)
                    .corner_radius(6.0)
                    .stroke(egui::Stroke::new(1.0, card_border))
                    .inner_margin(egui::Margin::same(16));
                let available_w = ui.available_width();
                if available_w > 500.0 {
                    let card_h = 120.0;
                    ui.columns(2, |cols| {
                        cols[0].set_min_height(card_h);
                        cols[0].set_max_height(card_h);
                        card_frame.clone().show(&mut cols[0], |ui| {
                            ui.set_min_height(card_h);
                            ui.set_max_height(card_h);
                            ui.strong("MCP 服务控制");
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                let start_btn = ui.add(egui::Button::new(
                                    egui::RichText::new("启动").color(egui::Color32::WHITE),
                                ).fill(
                                    if self.service_running {
                                        egui::Color32::from_rgb(40, 44, 56)
                                    } else {
                                        egui::Color32::from_rgb(40, 100, 60)
                                    },
                                ));
                                if start_btn.clicked() && !self.service_running {
                                    self.start_service();
                                }
                                let stop_btn = ui.add(egui::Button::new(
                                    egui::RichText::new("停止").color(egui::Color32::WHITE),
                                ).fill(
                                    if self.service_running {
                                        egui::Color32::from_rgb(140, 50, 50)
                                    } else {
                                        egui::Color32::from_rgb(40, 44, 56)
                                    },
                                ));
                                if stop_btn.clicked() && self.service_running {
                                    self.stop_service();
                                }
                                if ui.button("重启").clicked() {
                                    self.stop_service();
                                    self.start_service();
                                }
                                if ui.button("刷新状态").clicked() {
                                    self.refresh_status(ctx);
                                }
                            });
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label("MCP端口");
                                let resp = ui.add(
                                    egui::TextEdit::singleline(&mut self.port)
                                        .desired_width(65.0)
                                        .horizontal_align(egui::Align::Center),
                                );
                                if resp.changed() {
                                    self.port.retain(|c| c.is_ascii_digit());
                                    self.save_port_to_config();
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Web端口");
                                let resp = ui.add(
                                    egui::TextEdit::singleline(&mut self.web_port)
                                        .desired_width(65.0)
                                        .horizontal_align(egui::Align::Center),
                                );
                                if resp.changed() {
                                    self.web_port.retain(|c| c.is_ascii_digit());
                                    self.save_port_to_config();
                                }
                            });
                        });
                        cols[1].set_min_height(card_h);
                        cols[1].set_max_height(card_h);
                        card_frame.clone().show(&mut cols[1], |ui| {
                            ui.set_min_height(card_h);
                            ui.set_max_height(card_h);
                            ui.strong("系统设置");
                            ui.add_space(8.0);
                            if ui.checkbox(&mut self.auto_start, "开机自动启动").changed() {
                                self.toggle_auto_start();
                            }
                            if ui.checkbox(&mut self.use_tray, "启用系统托盘").changed() {
                                let exe_dir = std::env::current_exe()
                                    .ok()
                                    .and_then(|e| e.parent().map(|p| p.to_path_buf()))
                                    .unwrap_or_default();
                                file_ops::save_config_use_tray(&exe_dir, self.use_tray);
                                if self.use_tray && self.tray_icon.is_none() {
                                    if let Ok((icon, open_id, exit_id)) = create_tray_icon() {
                                        self.tray_icon = Some(icon);
                                        self.tray_open_id = open_id;
                                        self.tray_exit_id = exit_id;
                                    }
                                } else if !self.use_tray {
                                    self.tray_icon = None;
                                }
                            }
                            let mut dark_checked = self.dark_mode_override.unwrap_or(false);
                            if ui.checkbox(&mut dark_checked, "夜间模式").changed() {
                                self.dark_mode_override = Some(dark_checked);
                                let exe_dir = std::env::current_exe()
                                    .ok()
                                    .and_then(|e| e.parent().map(|p| p.to_path_buf()))
                                    .unwrap_or_default();
                                file_ops::save_config_dark_mode_override(&exe_dir, self.dark_mode_override);
                            }
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if ui
                                    .add_enabled(
                                        self.service_running,
                                        egui::Button::new("网页管理"),
                                    )
                                    .clicked()
                                {
                                    self.open_browser_web_admin();
                                }
                                if ui
                                    .add_enabled(
                                        self.service_running,
                                        egui::Button::new("关于"),
                                    )
                                    .clicked()
                                {
                                    self.open_browser_about();
                                }
                            });
                        });
                    });
                } else {
                    card_frame.clone().show(ui, |ui| {
                        ui.strong("MCP 服务控制");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let start_btn =
                                ui.add(egui::Button::new(
                                    egui::RichText::new("启动").color(egui::Color32::WHITE),
                                ).fill(if self.service_running {
                                    egui::Color32::from_rgb(40, 44, 56)
                                } else {
                                    egui::Color32::from_rgb(40, 100, 60)
                                }));
                            if start_btn.clicked() && !self.service_running {
                                self.start_service();
                            }
                            let stop_btn =
                                ui.add(egui::Button::new(
                                    egui::RichText::new("停止").color(egui::Color32::WHITE),
                                ).fill(if self.service_running {
                                    egui::Color32::from_rgb(140, 50, 50)
                                } else {
                                    egui::Color32::from_rgb(40, 44, 56)
                                }));
                            if stop_btn.clicked() && self.service_running {
                                self.stop_service();
                            }
                            if ui.button("重启").clicked() {
                                self.stop_service();
                                self.start_service();
                            }
                            if ui.button("刷新状态").clicked() {
                                self.refresh_status(ctx);
                            }
                        });
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("MCP端口：");
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.port)
                                    .desired_width(80.0)
                                    .horizontal_align(egui::Align::Center),
                            );
                            if resp.changed() {
                                self.port.retain(|c| c.is_ascii_digit());
                                self.save_port_to_config();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Web端口：");
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.web_port)
                                    .desired_width(80.0)
                                    .horizontal_align(egui::Align::Center),
                            );
                            if resp.changed() {
                                self.web_port.retain(|c| c.is_ascii_digit());
                                self.save_port_to_config();
                            }
                        });
                    });
                    ui.add_space(12.0);
                    card_frame.show(ui, |ui| {
                        ui.strong("系统设置");
                        ui.add_space(8.0);
                        if ui.checkbox(&mut self.auto_start, "开机自动启动").changed() {
                            self.toggle_auto_start();
                        }
                        if ui.checkbox(&mut self.use_tray, "启用系统托盘").changed() {
                            let exe_dir = std::env::current_exe()
                                .ok()
                                .and_then(|e| e.parent().map(|p| p.to_path_buf()))
                                .unwrap_or_default();
                            file_ops::save_config_use_tray(&exe_dir, self.use_tray);
                            if self.use_tray && self.tray_icon.is_none() {
                                if let Ok((icon, open_id, exit_id)) = create_tray_icon() {
                                    self.tray_icon = Some(icon);
                                    self.tray_open_id = open_id;
                                    self.tray_exit_id = exit_id;
                                }
                            } else if !self.use_tray {
                                self.tray_icon = None;
                            }
                        }
                        let mut dark_checked = self.dark_mode_override.unwrap_or(false);
                        if ui.checkbox(&mut dark_checked, "夜间模式").changed() {
                            self.dark_mode_override = Some(dark_checked);
                            let exe_dir = std::env::current_exe()
                                .ok()
                                .and_then(|e| e.parent().map(|p| p.to_path_buf()))
                                .unwrap_or_default();
                            file_ops::save_config_dark_mode_override(&exe_dir, self.dark_mode_override);
                        }
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(
                                    self.service_running,
                                    egui::Button::new("网页管理"),
                                )
                                .clicked()
                            {
                                self.open_browser_web_admin();
                            }
                            if ui
                                .add_enabled(
                                    self.service_running,
                                    egui::Button::new("关于"),
                                )
                                .clicked()
                            {
                                self.open_browser_about();
                            }
                        });
                    });
                };

                ui.add_space(12.0);

                egui::Frame::NONE
                    .fill(card_bg)
                    .corner_radius(6.0)
                    .stroke(egui::Stroke::new(1.0, card_border))
                    .inner_margin(egui::Margin::same(16))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.strong("Skill 管理");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("刷新").clicked() {
                                        self.refresh_skills();
                                    }
                                    if ui.button("导入Skill").clicked() {
                                        self.show_import_window = true;
                                        self.popup_close_flag.set(false);
                                        self.import_log.borrow_mut().clear();
                                        self.imported_count = 0;
                                        self.import_name.clear();
                                        self.import_tags.clear();
                                        self.import_note.clear();
                                        self.import_save_as_skill = false;
                                    }
                                },
                            );
                        });
                        ui.add_space(8.0);

                        let count = self.skills.len();
                        ui.label(format!("全部技能（共{}个）", count));
                        ui.add_space(6.0);

                        if count == 0 {
                            ui.label(egui::RichText::new("暂无技能，点击「导入Skill」添加").weak());
                        } else {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("技能名称").strong().size(12.0),
                                        );
                                    });
                                    ui.add_space(2.0);
                                    ui.separator();
                                    ui.add_space(2.0);

                                    for idx in 0..self.skills.len() {
                                        let skill = &self.skills[idx];
                                        let skill_name = skill.name.clone();
                                        let skill_desc: String =
                                            skill.description.chars().take(60).collect();
                                        let skill_desc =
                                            if skill_desc.len() < skill.description.len() {
                                                format!("{}...", skill_desc)
                                            } else if skill_desc.is_empty() {
                                                "无描述".to_string()
                                            } else {
                                                skill_desc
                                            };

                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&skill_name)
                                                        .strong()
                                                        .size(13.0),
                                                );
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        let del_btn = ui.add(
                                                            egui::Button::new(
                                                                egui::RichText::new("删除").color(egui::Color32::WHITE),
                                                            )
                                                                .fill(egui::Color32::from_rgb(
                                                                    140, 50, 50,
                                                                ))
                                                                .corner_radius(3.0),
                                                        );
                                                        if del_btn.clicked() {
                                                            self.show_delete_confirm = true;
                                                            self.delete_target = skill_name.clone();
                                                        }
                                                        let enabled = self.skills[idx].enabled;
                                                        let (bg, label, text_color) = if enabled {
                                                            (
                                                                egui::Color32::from_rgb(
                                                                    40, 100, 60,
                                                                ),
                                                                "启用",
                                                                egui::Color32::WHITE,
                                                            )
                                                        } else {
                                                            (
                                                                egui::Color32::from_rgb(80, 80, 80),
                                                                "禁用",
                                                                egui::Color32::from_rgb(
                                                                    180, 180, 180,
                                                                ),
                                                            )
                                                        };
                                                        let switch_btn = ui.add(
                                                            egui::Button::new(
                                                                egui::RichText::new(label)
                                                                    .color(text_color)
                                                                    .size(11.0),
                                                            )
                                                            .fill(bg)
                                                            .corner_radius(10.0)
                                                            .min_size(egui::vec2(48.0, 22.0)),
                                                        );
                                                        if switch_btn.clicked() {
                                                            let new_enabled = !enabled;
                                                            self.skills[idx].enabled = new_enabled;
                                                            if new_enabled {
                                                                let _ = api_client::enable_skill(&skill_name);
                                                            } else {
                                                                let _ = api_client::disable_skill(&skill_name);
                                                            }
                                                        }
                                                    },
                                                );
                                            });
                                            ui.label(
                                                egui::RichText::new(&skill_desc).small().weak(),
                                            );
                                        });
                                        ui.add_space(4.0);
                                        if idx < count - 1 {
                                            ui.separator();
                                            ui.add_space(4.0);
                                        }
                                    }
                                });
                        }
                    });

                ui.add_space(12.0);
            });
        });

        if self.show_delete_confirm {
            let mut confirmed = false;
            let mut cancelled = false;
            egui::Window::new("")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .fixed_size([280.0, 160.0])
                .frame({
                    let (dlg_bg, dlg_border) = if is_dark {
                        (egui::Color32::from_rgb(32, 35, 42), egui::Color32::from_rgb(60, 65, 78))
                    } else {
                        (egui::Color32::from_rgb(248, 248, 250), egui::Color32::from_rgb(180, 180, 185))
                    };
                    egui::Frame::NONE
                        .fill(dlg_bg)
                        .corner_radius(6.0)
                        .stroke(egui::Stroke::new(1.0, dlg_border))
                        .inner_margin(egui::Margin::same(0))
                })
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(16.0);
                        ui.label(egui::RichText::new("确认删除").size(16.0).strong());
                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "确定要删除技能「{}」吗？",
                                self.delete_target
                            ))
                            .size(14.0),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("删除后可通过管理后台恢复")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(200, 170, 80)),
                        );
                        ui.add_space(24.0);
                        ui.columns(2, |cols| {
                            cols[0].vertical_centered(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(egui::RichText::new("取消").size(13.0))
                                            .fill(egui::Color32::from_rgb(50, 54, 64))
                                            .corner_radius(4.0)
                                            .min_size(egui::vec2(80.0, 30.0)),
                                    )
                                    .clicked()
                                {
                                    cancelled = true;
                                }
                            });
                            cols[1].vertical_centered(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("删除")
                                                .size(13.0)
                                                .color(egui::Color32::WHITE),
                                        )
                                        .fill(egui::Color32::from_rgb(160, 50, 50))
                                        .corner_radius(4.0)
                                        .min_size(egui::vec2(80.0, 30.0)),
                                    )
                                    .clicked()
                                {
                                    confirmed = true;
                                }
                            });
                        });
                        ui.add_space(16.0);
                    });
                });
            if confirmed {
                self.delete_skill(&self.delete_target.clone());
                self.show_delete_confirm = false;
                self.delete_target.clear();
            }
            if cancelled {
                self.show_delete_confirm = false;
                self.delete_target.clear();
            }
        }

        if self.show_import_window {
            let popup_id = egui::ViewportId::from_hash_of("import_skill_popup");
            let log = self.import_log.clone();
            let close_flag = self.popup_close_flag.clone();
            let skills_dir = self.skills_dir.clone();
            let import_pending_paths = self.import_pending_paths.clone();
            let import_refresh_flag = self.import_refresh_flag.clone();

            ctx.show_viewport_immediate(
                popup_id,
                egui::ViewportBuilder::default()
                    .with_title("导入 Skill")
                    .with_inner_size([550.0, 520.0])
                    .with_min_inner_size([400.0, 350.0])
                    .with_position([200.0, 150.0]),
                move |ctx, class| {
                    if class == egui::ViewportClass::Embedded {
                        return;
                    }
                    if close_flag.get() {
                        return;
                    }
                    if ctx.input(|i| i.viewport().close_requested()) {
                        close_flag.set(true);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    }

                    let dropped = ctx.input(|i| i.raw.dropped_files.clone());
                    let is_dragging = ctx.input(|i| !i.raw.hovered_files.is_empty());
                    if !dropped.is_empty() {
                        let mut pending = import_pending_paths.borrow_mut();
                        let mut log = log.borrow_mut();
                        for file in &dropped {
                            if let Some(path) = &file.path {
                                let name = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                let dest = skills_dir.join(&name);
                                if dest.exists() {
                                    log.push(format!("→ {} 已存在，跳过", name));
                                } else {
                                    pending.push(path.clone());
                                    log.push(format!("+ {} (待确认)", name));
                                }
                            }
                        }
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("导入 Skill");
                        ui.separator();

                        let drop_size = egui::vec2(ui.available_width(), 100.0);
                        let (drop_rect, drop_resp) =
                            ui.allocate_exact_size(drop_size, egui::Sense::hover());
                        let stroke_color = if is_dragging {
                            egui::Color32::from_rgb(80, 180, 120)
                        } else if drop_resp.hovered() {
                            egui::Color32::from_rgb(80, 120, 200)
                        } else {
                            egui::Color32::from_rgb(80, 80, 80)
                        };
                        ui.painter().rect_stroke(
                            drop_rect,
                            4.0,
                            (2.0, stroke_color),
                            egui::StrokeKind::Inside,
                        );
                        let hint = if is_dragging {
                            "释放以导入 Skill"
                        } else {
                            "可拖拽多个skill目录到此处批量导入"
                        };
                        ui.painter().text(
                            drop_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            hint,
                            egui::FontId::new(13.0, egui::FontFamily::Proportional),
                            egui::Color32::from_rgb(120, 120, 120),
                        );

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("目标目录：").small().weak());
                            ui.label(
                                egui::RichText::new(skills_dir.to_string_lossy().to_string())
                                    .small()
                                    .color(egui::Color32::from_rgb(100, 160, 220)),
                            );
                        });

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("选择单个文件").clicked() {
                                if let Some(p) = rfd::FileDialog::new().pick_file() {
                                    let path = p.to_string_lossy().to_string();
                                    let name = std::path::Path::new(&path)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown")
                                        .to_string();
                                    let dest = skills_dir.join(&name);
                                    if std::path::Path::new(&path).is_dir() && !dest.exists() {
                                        let _ = std::fs::copy(&path, &dest);
                                    }
                                }
                            }
                            if ui.button("选择文件夹批量导入").clicked() {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    let name = folder
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown")
                                        .to_string();
                                    let dest = skills_dir.join(&name);
                                    if !dest.exists() {
                                        let _ = file_ops::copy_dir_all(&folder, &dest);
                                    }
                                }
                            }
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("取消").clicked() {
                                        close_flag.set(true);
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                    if ui.button("确认导入").clicked() {
                                        let pending = import_pending_paths.borrow().clone();
                                        import_pending_paths.borrow_mut().clear();
                                        let mut log = log.borrow_mut();
                                        for path in &pending {
                                            let name = path.file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("unknown")
                                                .to_string();
                                            let dest = skills_dir.join(&name);
                                            if path.is_dir() {
                                                match file_ops::copy_skill_dir(path, &dest) {
                                                    Ok(_) => log.push(format!("✓ {} 导入成功", name)),
                                                    Err(e) => log.push(format!("✗ {} {}", name, e)),
                                                }
                                            } else {
                                                match std::fs::copy(path, &dest) {
                                                    Ok(_) => log.push(format!("✓ {} 导入成功", name)),
                                                    Err(e) => log.push(format!("✗ {} {}", name, e)),
                                                }
                                            }
                                        }
                                        drop(log);
                                        let _ = api_client::refresh_skills();
                                        import_refresh_flag.set(true);
                                    }
                                },
                            );
                        });

                        ui.add_space(8.0);
                        ui.separator();

                        ui.strong("导入日志：");
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                let log_ref = log.borrow();
                                for entry in log_ref.iter().rev().take(20) {
                                    ui.label(entry);
                                }
                            });
                    });
                },
            );

            if self.popup_close_flag.get() {
                self.show_import_window = false;
                self.popup_close_flag.set(false);
            }
        }
    }
}

fn create_tray_icon()
-> Result<(tray_icon::TrayIcon, muda::MenuId, muda::MenuId), Box<dyn std::error::Error>> {
    use tray_icon::TrayIconBuilder;
    use tray_icon::menu::{Menu, MenuItem};

    let menu = Menu::new();
    let open_item = MenuItem::with_id("open", "打开窗口", true, None);
    let exit_item = MenuItem::with_id("exit", "退出", true, None);
    let open_id = open_item.id().clone();
    let exit_id = exit_item.id().clone();
    menu.append(&open_item)?;
    menu.append(&exit_item)?;

    let icon = load_icon_from_bytes();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("AI 技能百宝箱")
        .with_icon(icon)
        .build()?;

    Ok((tray_icon, open_id, exit_id))
}

fn load_icon_from_bytes() -> tray_icon::Icon {
    let bytes = include_bytes!("..\\assets\\logo.png");
    let img = image::load_from_memory(bytes).expect("加载图标失败")
        .resize(16, 16, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let (w, h) = img.dimensions();
    let rgba = img.into_raw();
    tray_icon::Icon::from_rgba(rgba, w, h).expect("创建图标失败")
}

fn load_window_icon() -> egui::IconData {
    let bytes = include_bytes!("..\\assets\\logo.png");
    let img = image::load_from_memory(bytes).expect("加载图标失败").to_rgba8();
    let (w, h) = img.dimensions();
    let rgba = img.into_raw();
    egui::IconData { rgba, width: w, height: h }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq AISkillBox.exe", "/NH"])
            .creation_flags(0x08000000)
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let count = stdout.lines()
            .filter(|line| line.contains("AISkillBox.exe"))
            .count();
        if count > 1 {
            eprintln!("GUI 已在运行中，禁止双开");
            std::process::exit(1);
        }
    }

    let window_icon = load_window_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 550.0])
            .with_min_inner_size([400.0, 400.0])
            .with_icon(window_icon),
        ..Default::default()
    };

    eframe::run_native(
        "AI 技能百宝箱",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            let font_path = "C:\\Windows\\Fonts\\msyh.ttc";
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "chinese".to_owned(),
                    egui::FontData::from_owned(font_data).into(),
                );
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "chinese".to_owned());
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "chinese".to_owned());
            }
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(App::new(cc)))
        }),
    )?;
    Ok(())
}
