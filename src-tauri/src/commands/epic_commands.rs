use crate::utils::epic_api::{self, EpicApi};

#[tauri::command]
pub fn get_epic_auth_url() -> String {
    EpicApi::get_auth_url()
}

#[tauri::command]
pub async fn epic_login_with_code(code: String) -> Result<(), String> {
    let code = code.trim().replace('"', "");
    let api = EpicApi::new();
    let session = api.login_with_auth_code(&code).await?;
    epic_api::save_session(&session).await?;
    Ok(())
}

#[tauri::command]
pub async fn epic_try_restore_session() -> Result<bool, String> {
    let Some(saved) = epic_api::load_session().await else {
        return Ok(false);
    };

    let api = EpicApi::new();
    match api.refresh_session(&saved.refresh_token).await {
        Ok(session) => {
            epic_api::save_session(&session).await?;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn epic_logout() -> Result<(), String> {
    epic_api::clear_session().await
}

#[tauri::command]
pub async fn epic_is_logged_in() -> Result<bool, String> {
    Ok(epic_api::load_session().await.is_some())
}
