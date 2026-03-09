use anyhow::Result;
use rusqlite::{OptionalExtension, params};

use crate::models::AppConfig;

use super::{Database, shared::now_utc_string};

impl Database {
    pub fn load_app_config(&self) -> Result<AppConfig> {
        Ok(AppConfig {
            azure_devops_org_url: self
                .get_setting("azure_devops_org_url")?
                .unwrap_or_default(),
            azure_devops_project: self
                .get_setting("azure_devops_project")?
                .unwrap_or_default(),
            azure_devops_pat: self.get_setting("azure_devops_pat")?.unwrap_or_default(),
            outlook_enabled: self
                .get_setting("outlook_enabled")?
                .map(|value| value == "true")
                .unwrap_or(true),
            azure_devops_enabled: self
                .get_setting("azure_devops_enabled")?
                .map(|value| value == "true")
                .unwrap_or(false),
            minimize_to_tray: self
                .get_setting("minimize_to_tray")?
                .map(|value| value == "true")
                .unwrap_or(true),
            activity_poll_seconds: self
                .get_setting("activity_poll_seconds")?
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(2),
            idle_threshold_minutes: self
                .get_setting("idle_threshold_minutes")?
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(5),
        })
    }

    pub fn save_app_config(&self, config: &AppConfig) -> Result<()> {
        self.set_setting("azure_devops_org_url", &config.azure_devops_org_url)?;
        self.set_setting("azure_devops_project", &config.azure_devops_project)?;
        self.set_setting("azure_devops_pat", &config.azure_devops_pat)?;
        self.set_setting(
            "outlook_enabled",
            if config.outlook_enabled {
                "true"
            } else {
                "false"
            },
        )?;
        self.set_setting(
            "azure_devops_enabled",
            if config.azure_devops_enabled {
                "true"
            } else {
                "false"
            },
        )?;
        self.set_setting(
            "minimize_to_tray",
            if config.minimize_to_tray {
                "true"
            } else {
                "false"
            },
        )?;
        self.set_setting(
            "activity_poll_seconds",
            &config.activity_poll_seconds.to_string(),
        )?;
        self.set_setting(
            "idle_threshold_minutes",
            &config.idle_threshold_minutes.to_string(),
        )?;
        Ok(())
    }

    fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()
            .map_err(anyhow::Error::from)
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "
            INSERT INTO app_settings (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
            ",
            params![key, value, now_utc_string()],
        )?;
        Ok(())
    }
}
