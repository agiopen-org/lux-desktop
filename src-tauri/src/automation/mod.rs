mod session;
mod state;
mod types;

use crate::error::Result;
use session::Session;
use std::sync::Arc;
use tauri::{AppHandle, Window};
use tokio::sync::Mutex;

pub use state::AutomationState;

#[derive(Default)]
pub struct AutomationEngine {
  session: Option<Arc<Mutex<Session>>>,
}

impl AutomationEngine {
  pub async fn start_session(
    &mut self,
    app: AppHandle,
    window: Window,
    session_id: String,
    instruction: String,
    mode: String,
  ) -> Result<()> {
    self.stop_session().await?;
    self.session = Some(Session::new(app, window, session_id, instruction, mode).await?);
    Ok(())
  }

  pub async fn stop_session(&mut self) -> Result<()> {
    if let Some(session) = self.session.take() {
      session.lock_owned().await.cancel().await?;
    }
    Ok(())
  }

  pub async fn get_state(&self) -> Option<AutomationState> {
    match self.session.as_ref() {
      Some(session) => Some(session.lock().await.get_state().await),
      None => None,
    }
  }
}
