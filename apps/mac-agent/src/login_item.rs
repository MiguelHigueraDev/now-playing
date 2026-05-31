use auto_launch::AutoLaunch;

const APP_NAME: &str = "Now Playing";

pub fn manager() -> AutoLaunch {
    AutoLaunch::new(APP_NAME, &app_path(), true, &[] as &[&str])
}

pub fn is_enabled() -> anyhow::Result<bool> {
    manager()
        .is_enabled()
        .map_err(|err| anyhow::anyhow!("failed to read login item state: {err}"))
}

pub fn enable() -> anyhow::Result<()> {
    manager()
        .enable()
        .map_err(|err| anyhow::anyhow!("failed to enable login item: {err}"))
}

pub fn disable() -> anyhow::Result<()> {
    manager()
        .disable()
        .map_err(|err| anyhow::anyhow!("failed to disable login item: {err}"))
}

pub fn toggle() -> anyhow::Result<bool> {
    if is_enabled()? {
        disable()?;
        Ok(false)
    } else {
        enable()?;
        Ok(true)
    }
}

fn app_path() -> String {
    let Ok(exe) = std::env::current_exe() else {
        return String::new();
    };

    if crate::is_app_bundle() {
        if let Some(app) = exe
            .parent()
            .and_then(|macos| macos.parent())
            .and_then(|contents| contents.parent())
        {
            return app.to_string_lossy().to_string();
        }
    }

    exe.to_string_lossy().to_string()
}
