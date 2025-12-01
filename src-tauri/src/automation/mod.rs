mod event;
mod state;
mod types;

use crate::error::{LuxDesktopError, Result};
use enigo::{Button, Enigo, Settings};
use futures_util::FutureExt;
use rust_socketio::{
  TransportType,
  asynchronous::{Client, ClientBuilder},
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Window};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

pub use state::{AutomationState, AutomationStatus};

const MODE_THINKER: &str = "thinker";
const MODEL_ACTOR: &str = "lux-actor-1";
const MODEL_THINKER: &str = "lux-thinker-1";

pub struct Session {
  pub socket: Client,
  pub state: Arc<Mutex<state::AutomationState>>,
}

#[derive(Default)]
pub struct AutomationEngine {
  session: Option<Session>,
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
    self.stop_session(app.clone()).await?;
    let state = Arc::new(Mutex::new(AutomationState::new(
      session_id.clone(),
      instruction.clone(),
    )));
    event::on_state_update(&app, state.lock().await)?;
    let monitor = window
      .current_monitor()?
      .ok_or_else(LuxDesktopError::error_current_monitor)?;
    let scale_factor = monitor.scale_factor();
    let pos = monitor.position().cast::<f64>();
    let get_coordinate = {
      let (offset_x, offset_y) = (pos.x / scale_factor, pos.y / scale_factor);
      let size = monitor.size().cast::<f64>();
      let (size_x, size_y) = (size.width / scale_factor, size.height / scale_factor);
      move |x: usize, y: usize| {
        (
          (x as f64 / 1000.0 * size_x + offset_x) as i32,
          (y as f64 / 1000.0 * size_y + offset_y) as i32,
        )
      }
    };
    let (x, y) = (pos.x / scale_factor, pos.y / scale_factor);

    let store = app.store("settings.json")?;
    let base_url = store
      .get("baseUrl")
      .and_then(|base_url| base_url.as_str().map(|base_url| base_url.to_string()))
      .unwrap_or_else(|| "http://127.0.0.1:8000".into());
    let mut socket = ClientBuilder::new(base_url)
      .namespace(format!("/session/{session_id}"))
      .transport_type(TransportType::Websocket);
    {
      let app = app.clone();
      let state = state.clone();
      let instruction = instruction.clone();
      let mode = mode.clone();
      let model: String = if mode == MODE_THINKER {
        MODEL_THINKER
      } else {
        MODEL_ACTOR
      }
      .into();
      socket = socket.on("open", move |_, client| {
        event::on_open(
          app.clone(),
          state.clone(),
          client,
          instruction.clone(),
          mode.clone(),
          model.clone(),
          None,
        )
        .boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      socket = socket.on_with_ack("request_screenshot", move |payload, client, ack| {
        event::on_request_screenshot(
          app.clone(),
          state.clone(),
          payload,
          client,
          ack,
          x,
          y,
        )
        .boxed()
      });
    }
    let enigo = Arc::new(tokio::sync::Mutex::new(Enigo::new(&Settings::default())?));
    {
      let app = app.clone();
      let state = state.clone();
      let enigo = enigo.clone();
      socket = socket.on_with_ack("click", move |payload, client, ack| {
        event::on_click(
          app.clone(),
          state.clone(),
          get_coordinate,
          enigo.clone(),
          payload,
          Button::Left,
          1,
          client,
          ack,
        )
        .boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      let enigo = enigo.clone();
      socket = socket.on_with_ack("left_double", move |payload, client, ack| {
        event::on_click(
          app.clone(),
          state.clone(),
          get_coordinate,
          enigo.clone(),
          payload,
          Button::Left,
          2,
          client,
          ack,
        )
        .boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      let enigo = enigo.clone();
      socket = socket.on_with_ack("left_triple", move |payload, client, ack| {
        event::on_click(
          app.clone(),
          state.clone(),
          get_coordinate,
          enigo.clone(),
          payload,
          Button::Left,
          3,
          client,
          ack,
        )
        .boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      let enigo = enigo.clone();
      socket = socket.on_with_ack("right_single", move |payload, client, ack| {
        event::on_click(
          app.clone(),
          state.clone(),
          get_coordinate,
          enigo.clone(),
          payload,
          Button::Right,
          1,
          client,
          ack,
        )
        .boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      let enigo = enigo.clone();
      socket = socket.on_with_ack("drag", move |payload, client, ack| {
        event::on_drag(
          app.clone(),
          state.clone(),
          get_coordinate,
          enigo.clone(),
          payload,
          client,
          ack,
        )
        .boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      let enigo = enigo.clone();
      socket = socket.on_with_ack("hotkey", move |payload, client, ack| {
        event::on_hotkey(
          app.clone(),
          state.clone(),
          enigo.clone(),
          payload,
          client,
          ack,
        )
        .boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      let enigo = enigo.clone();
      socket = socket.on_with_ack("type", move |payload, client, ack| {
        event::on_type(
          app.clone(),
          state.clone(),
          enigo.clone(),
          payload,
          client,
          ack,
        )
        .boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      let enigo = enigo.clone();
      socket = socket.on_with_ack("scroll", move |payload, client, ack| {
        event::on_scroll(
          app.clone(),
          state.clone(),
          get_coordinate,
          enigo.clone(),
          payload,
          client,
          ack,
        )
        .boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      socket = socket.on_with_ack("wait", move |payload, client, ack| {
        event::on_wait(app.clone(), state.clone(), payload, client, ack).boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      socket = socket.on("finish", move |_, _| {
        event::on_finish(app.clone(), state.clone()).boxed()
      });
    }
    {
      let app = app.clone();
      let state = state.clone();
      socket = socket.on("error", move |payload, _| {
        event::on_error(app.clone(), state.clone(), payload).boxed()
      });
    }
    let socket = socket.connect().await?;

    self.session = Some(Session { socket, state });
    Ok(())
  }

  pub async fn stop_session(&mut self, app: AppHandle) -> Result<()> {
    if let Some(session) = self.session.take() {
      session.socket.disconnect().await?;
      let mut state = session.state.lock().await;
      state.status = AutomationStatus::Cancelled;
      app.emit("stateUpdated", serde_json::to_value(state.clone())?)?;
    }
    Ok(())
  }

  pub async fn get_state(&self) -> Option<AutomationState> {
    match self.session.as_ref() {
      Some(session) => Some(session.state.lock().await.clone()),
      None => None,
    }
  }
}
