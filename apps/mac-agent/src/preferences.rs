use egui::{Align, CentralPanel, Context, Layout, RichText, TextEdit, Ui};

use crate::config::AgentConfig;

pub struct PreferencesState {
    pub api_base_url: String,
    pub auth_token: String,
    pub poll_interval_secs: String,
    pub validation_error: Option<String>,
    pub saved_message: Option<String>,
    pub visible: bool,
}

impl PreferencesState {
    pub fn from_config(config: &AgentConfig) -> Self {
        Self {
            api_base_url: config.api_base_url.clone(),
            auth_token: config.auth_token.clone(),
            poll_interval_secs: config.poll_interval_secs.to_string(),
            validation_error: None,
            saved_message: None,
            visible: false,
        }
    }

    pub fn open(&mut self, config: &AgentConfig) {
        *self = Self::from_config(config);
        self.visible = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.validation_error = None;
        self.saved_message = None;
    }

    pub fn build_config(&self) -> anyhow::Result<AgentConfig> {
        let poll_interval_secs: u64 = self
            .poll_interval_secs
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("poll interval must be a number"))?;

        let config = AgentConfig {
            api_base_url: self.api_base_url.trim().to_string(),
            auth_token: self.auth_token.trim().to_string(),
            poll_interval_secs,
        };

        config.validate()?;
        Ok(config)
    }

    pub fn ui(&mut self, ctx: &Context) -> PreferencesAction {
        let mut action = PreferencesAction::None;

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Now Playing Preferences");
            ui.add_space(8.0);
            ui.label("Configure the API connection for the menu bar agent.");
            ui.add_space(12.0);

            form_field(ui, "API Base URL", &mut self.api_base_url, false);
            form_field(ui, "Auth Token", &mut self.auth_token, true);

            ui.horizontal(|ui| {
                ui.label("Poll Interval (seconds)");
                ui.add(
                    TextEdit::singleline(&mut self.poll_interval_secs)
                        .desired_width(60.0)
                        .hint_text("3"),
                );
                ui.label(RichText::new("Must be between 2 and 5").weak());
            });

            if let Some(error) = &self.validation_error {
                ui.add_space(8.0);
                ui.colored_label(egui::Color32::from_rgb(200, 60, 60), error);
            }

            if let Some(message) = &self.saved_message {
                ui.add_space(8.0);
                ui.colored_label(egui::Color32::from_rgb(40, 140, 70), message);
            }

            ui.add_space(16.0);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    action = PreferencesAction::Cancel;
                }

                if ui.button("Save").clicked() {
                    action = PreferencesAction::Save;
                }
            });
        });

        action
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesAction {
    None,
    Save,
    Cancel,
}

fn form_field(ui: &mut Ui, label: &str, value: &mut String, password: bool) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            TextEdit::singleline(value)
                .desired_width(f32::INFINITY)
                .password(password),
        );
    });
    ui.add_space(8.0);
}
