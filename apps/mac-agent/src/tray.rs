use std::sync::{Arc, Mutex};
use std::thread;

use egui_winit::winit::application::ApplicationHandler;
use egui_winit::winit::event::WindowEvent;
use egui_winit::winit::event_loop::{ActiveEventLoop, EventLoop};
use egui_winit::winit::window::WindowId;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, watch};
use tokio_util::sync::CancellationToken;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::config::AgentConfig;
use crate::config_store::ConfigStore;
use crate::gl_window::{create_gl_context, GlutinWindowContext};
use crate::login_item;
use crate::preferences::{PreferencesAction, PreferencesState};
use crate::{init_file_logging, run_agent, AgentStatus};

enum UserEvent {
    TrayIconEvent,
    MenuEvent(tray_icon::menu::MenuEvent),
}

struct PreferencesWindow {
    gl_window: GlutinWindowContext,
    gl: Arc<glow::Context>,
    egui_glow: egui_glow::EguiGlow,
}

struct AppState {
    config_store: ConfigStore,
    config_tx: watch::Sender<AgentConfig>,
    config_rx: watch::Receiver<AgentConfig>,
    preferences: PreferencesState,
    status: AgentStatus,
    login_enabled: bool,
    status_item: MenuItem,
    login_item: MenuItem,
    cancel: CancellationToken,
    menu_ids: MenuIds,
    /// Keeps the tray icon alive for the lifetime of the app.
    #[allow(dead_code)]
    tray_icon: Arc<Mutex<TrayIcon>>,
    preferences_window: Option<PreferencesWindow>,
}

struct MenuIds {
    preferences: MenuId,
    login: MenuId,
    quit: MenuId,
}

struct MacAgentApp {
    state: AppState,
    status_rx: mpsc::Receiver<AgentStatus>,
}

pub fn run_app() -> anyhow::Result<()> {
    let (config_store, config) = ConfigStore::load_or_create()?;
    init_file_logging(&config_store.log_dir())?;

    let (config_tx, config_rx) = watch::channel(config.clone());
    let (status_tx, status_rx) = mpsc::channel(32);
    let cancel = CancellationToken::new();
    let agent_cancel = cancel.clone();

    let agent_config_rx = config_rx.clone();
    thread::spawn(move || {
        let runtime = Runtime::new().expect("failed to create tokio runtime");
        runtime.block_on(async {
            if let Err(err) = run_agent(agent_config_rx, agent_cancel, status_tx).await {
                tracing::error!(error = %err, "agent task exited with error");
            }
        });
    });

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    TrayIconEvent::set_event_handler(Some(move |_event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent);
    }));

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let status_item = MenuItem::new("Status: Starting…", false, None);
    let preferences_item = MenuItem::with_id("preferences", "Preferences…", true, None);
    let login_item = MenuItem::with_id("login", "Enable at Login", true, None);
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);

    let tray_menu = Menu::new();
    tray_menu.append_items(&[
        &status_item,
        &PredefinedMenuItem::separator(),
        &preferences_item,
        &login_item,
        &PredefinedMenuItem::separator(),
        &quit_item,
    ])?;

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Now Playing Agent")
        .with_icon(load_tray_icon()?)
        .build()?;

    let login_enabled = login_item::is_enabled().unwrap_or(false);
    update_status_menu(&status_item, &AgentStatus::Idle);
    update_login_menu(&login_item, login_enabled);

    let menu_ids = MenuIds {
        preferences: preferences_item.id().clone(),
        login: login_item.id().clone(),
        quit: quit_item.id().clone(),
    };

    let mut app = MacAgentApp {
        state: AppState {
            config_store,
            config_tx,
            config_rx,
            preferences: PreferencesState::from_config(&config),
            status: AgentStatus::Idle,
            login_enabled,
            status_item,
            login_item,
            cancel,
            menu_ids,
            tray_icon: Arc::new(Mutex::new(tray_icon)),
            preferences_window: None,
        },
        status_rx,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}

impl ApplicationHandler<UserEvent> for MacAgentApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        while let Ok(status) = self.status_rx.try_recv() {
            self.state.status = status;
            update_status_menu(&self.state.status_item, &self.state.status);
        }

        match event {
            UserEvent::MenuEvent(menu_event) => {
                if menu_event.id == self.state.menu_ids.preferences {
                    self.open_preferences(event_loop);
                } else if menu_event.id == self.state.menu_ids.login {
                    match login_item::toggle() {
                        Ok(enabled) => {
                            self.state.login_enabled = enabled;
                            update_login_menu(&self.state.login_item, enabled);
                        }
                        Err(err) => {
                            self.state.status = AgentStatus::Error(err.to_string());
                            update_status_menu(&self.state.status_item, &self.state.status);
                        }
                    }
                } else if menu_event.id == self.state.menu_ids.quit {
                    self.state.cancel.cancel();
                    event_loop.exit();
                }
            }
            UserEvent::TrayIconEvent => {}
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        while let Ok(status) = self.status_rx.try_recv() {
            self.state.status = status;
            update_status_menu(&self.state.status_item, &self.state.status);
        }

        let Some(prefs) = self.state.preferences_window.as_mut() else {
            return;
        };

        if prefs.gl_window.window.id() != window_id {
            return;
        }

        if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
            self.state.preferences.close();
            self.state.preferences_window = None;
            return;
        }

        if let WindowEvent::Resized(physical_size) = event {
            prefs.gl_window.resize(physical_size);
        }

        let response = prefs
            .egui_glow
            .on_window_event(&prefs.gl_window.window, &event);

        if matches!(event, WindowEvent::RedrawRequested) {
            let mut action = PreferencesAction::None;
            prefs.egui_glow.run(&prefs.gl_window.window, |ctx| {
                action = self.state.preferences.ui(ctx);
            });

            unsafe {
                use glow::HasContext as _;
                prefs.gl.clear_color(0.12, 0.12, 0.14, 1.0);
                prefs.gl.clear(glow::COLOR_BUFFER_BIT);
            }

            prefs.egui_glow.paint(&prefs.gl_window.window);
            let _ = prefs.gl_window.swap_buffers();

            let mut close_preferences = false;
            match action {
                PreferencesAction::Save => match self.state.preferences.build_config() {
                    Ok(config) => {
                        if let Err(err) = self.state.config_store.save(&config) {
                            self.state.preferences.validation_error = Some(err.to_string());
                        } else if self.state.config_tx.send(config).is_err() {
                            self.state.preferences.validation_error =
                                Some("Agent is not running".to_string());
                        } else {
                            self.state.preferences.validation_error = None;
                            self.state.preferences.saved_message = Some("Settings saved".to_string());
                        }
                    }
                    Err(err) => {
                        self.state.preferences.validation_error = Some(err.to_string());
                    }
                },
                PreferencesAction::Cancel => {
                    self.state.preferences.close();
                    close_preferences = true;
                }
                PreferencesAction::None => {}
            }

            if close_preferences {
                self.state.preferences_window = None;
                return;
            }
        }

        if response.repaint {
            prefs.gl_window.window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        while let Ok(status) = self.status_rx.try_recv() {
            self.state.status = status;
            update_status_menu(&self.state.status_item, &self.state.status);
        }

        if let Some(prefs) = self.state.preferences_window.as_ref() {
            prefs.gl_window.window.request_redraw();
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(mut prefs) = self.state.preferences_window.take() {
            prefs.egui_glow.destroy();
        }
    }
}

impl MacAgentApp {
    fn open_preferences(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.preferences_window.is_some() {
            if let Some(prefs) = self.state.preferences_window.as_ref() {
                prefs.gl_window.window.focus_window();
            }
            return;
        }

        self.state
            .preferences
            .open(&self.state.config_rx.borrow());

        let gl_window = GlutinWindowContext::new(
            event_loop,
            "Now Playing Preferences",
            460.0,
            320.0,
        );
        let gl = create_gl_context(&gl_window);
        let egui_glow = egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true);

        gl_window.window.set_visible(true);
        gl_window.window.request_redraw();

        self.state.preferences_window = Some(PreferencesWindow {
            gl_window,
            gl,
            egui_glow,
        });
    }
}

fn update_status_menu(item: &MenuItem, status: &AgentStatus) {
    let _ = item.set_text(status.menu_label());
}

fn update_login_menu(item: &MenuItem, enabled: bool) {
    let label = if enabled {
        "Disable at Login"
    } else {
        "Enable at Login"
    };
    let _ = item.set_text(label);
}

fn load_tray_icon() -> anyhow::Result<Icon> {
    use image::GenericImageView;

    let image = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .map_err(|err| anyhow::anyhow!("failed to load tray icon: {err}"))?
        .into_rgba8();

    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height)
        .map_err(|err| anyhow::anyhow!("failed to build tray icon: {err}"))
}
