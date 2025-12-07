use crate::automation::{AutomationEngine, AutomationState};
use tauri::{AppHandle, Manager, Window};
use tokio::sync::Mutex;

#[tauri::command]
pub async fn start_session(
  app: AppHandle,
  window: Window,
  session_id: String,
  instruction: String,
  mode: String,
) -> Result<(), String> {
  app
    .state::<Mutex<AutomationEngine>>()
    .lock()
    .await
    .start_session(app.clone(), window, session_id, instruction, mode)
    .await
    .map_err(|err| format!("{err:?}"))
}

#[tauri::command]
pub async fn stop_session(app: AppHandle) -> Result<(), String> {
  app
    .state::<Mutex<AutomationEngine>>()
    .lock()
    .await
    .stop_session()
    .await
    .map_err(|err| format!("{err:?}"))
}

#[tauri::command]
pub async fn get_state(app: AppHandle) -> Option<AutomationState> {
  app
    .state::<Mutex<AutomationEngine>>()
    .lock()
    .await
    .get_state()
    .await
}
