use crate::utils::epic_api::{self, EpicApi, EpicSession};
use std::sync::Mutex;
use tauri::State;

pub struct EpicState {
    pub session: Mutex<Option<EpicSession>>,
}

#[tauri::command]
pub fn get_epic_auth_url() -> String {
    EpicApi::get_auth_url()
}

#[tauri::command]
pub async fn epic_login_with_code(code: String, state: State<'_, EpicState>) -> Result<(), String> {
    let code = code.trim().replace('"', "");
    let api = EpicApi::new();
    let session = api.login_with_auth_code(&code).await?;

    epic_api::save_session(&session).await?;
    *state.session.lock().unwrap() = Some(session);

    Ok(())
}

#[tauri::command]
pub async fn epic_try_restore_session(state: State<'_, EpicState>) -> Result<bool, String> {
    let Some(saved) = epic_api::load_session().await else {
        return Ok(false);
    };

    let api = EpicApi::new();

    // Try refresh token first
    match api.refresh_session(&saved.refresh_token).await {
        Ok(session) => {
            epic_api::save_session(&session).await?;
            *state.session.lock().unwrap() = Some(session);
            return Ok(true);
        }
        Err(_) => {}
    }

    Ok(false)
}

#[tauri::command]
pub async fn epic_logout(state: State<'_, EpicState>) -> Result<(), String> {
    epic_api::clear_session().await?;
    *state.session.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub async fn epic_is_logged_in(state: State<'_, EpicState>) -> Result<bool, String> {
    Ok(state.session.lock().unwrap().is_some())
}
