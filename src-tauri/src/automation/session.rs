use super::{state::{Action, AutomationState, AutomationStatus}, types};
use crate::error::{LuxDesktopError, Result};
use base64::Engine;
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use futures_util::FutureExt;
use image::{DynamicImage, ImageFormat, codecs::jpeg::JpegEncoder, imageops::FilterType};
use rust_socketio::{Event, Payload, TransportType, asynchronous::{Client, ClientBuilder}};
use serde_json::json;
use tauri::{AppHandle, Emitter, Window};
use tauri_plugin_http::reqwest;
use std::{io::Cursor, sync::Arc, time::Duration};
use tauri_plugin_store::StoreExt;
use tokio::{sync::{Mutex, MutexGuard}, time::sleep};

const MODE_THINKER: &str = "thinker";
const MODEL_ACTOR: &str = "lux-actor-1";
const MODEL_THINKER: &str = "lux-thinker-1";

fn from_payload<T: serde::de::DeserializeOwned>(payload: Payload) -> Result<T> {
  if let Payload::Text(mut payload) = payload {
    if payload.len() != 1 {
      Err(LuxDesktopError::InvalidPayload(format!(
        "Expected 1 payload, got {}",
        payload.len()
      )))
    } else {
      serde_json::from_value::<T>(payload.pop().unwrap())
        .map_err(|err| LuxDesktopError::InvalidPayload(format!("Error parsing payload: {err:?}")))
    }
  } else {
    Err(LuxDesktopError::InvalidPayload(format!(
      "Expected `Payload::Text`, got {payload:?}"
    )))
  }
}

pub struct Session {
  app: AppHandle,
  enigo: Arc<Mutex<enigo::Enigo>>,
  socket: Option<Client>,

  offset_x: f64,
  offset_y: f64,
  size_x: f64,
  size_y: f64,
  x: f64,
  y: f64,

  state: Arc<Mutex<AutomationState>>,
  abandoned: bool,
}

impl Session {
  pub async fn new(
    app: AppHandle,
    window: Window,
    session_id: String,
    instruction: String,
    mode: String,
  ) -> Result<Arc<Mutex<Self>>> {
    let state = Arc::new(Mutex::new(AutomationState::new(
      session_id.clone(),
      instruction.clone(),
    )));

    let monitor = window
      .current_monitor()?
      .ok_or_else(LuxDesktopError::error_current_monitor)?;
    let scale_factor = monitor.scale_factor();
    let pos = monitor.position().cast::<f64>();
    let (offset_x, offset_y) = (pos.x / scale_factor, pos.y / scale_factor);
    let size = monitor.size().cast::<f64>();
    let (size_x, size_y) = (size.width / scale_factor, size.height / scale_factor);
    let (x, y) = (pos.x / scale_factor, pos.y / scale_factor);

    let store = app.store("settings.json")?;
    let base_url = store
      .get("baseUrl")
      .and_then(|base_url| base_url.as_str().map(|base_url| base_url.to_string()))
      .unwrap_or_else(|| "http://127.0.0.1:8000".into());

    let session = Self {
      app,
      enigo: Arc::new(tokio::sync::Mutex::new(Enigo::new(&Settings::default())?)),
      socket: None,
      offset_x,
      offset_y,
      size_x,
      size_y,
      x,
      y,
      state,
      abandoned: false,
    };
    session.on_state_update(session.state.lock().await)?;
    let session = Arc::new(Mutex::new(session));
    let mut builder = ClientBuilder::new(base_url)
        .namespace(format!("/session/{session_id}"))
        .transport_type(TransportType::Websocket);
    {
      let session = session.clone();
      builder = builder
        .on("open", move |_, _| {
          let session = session.clone();
          let instruction = instruction.clone();
          let mode = mode.clone();
          async move {
            let session = session.lock().await;
            if let Err(err) = session.on_open(instruction, mode).await {
              session.on_automation_error(&err).await;
            }
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("request_screenshot", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_request_screenshot(payload).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("click", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_click(payload, Button::Left, 1).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("left_double", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_click(payload, Button::Left, 2).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("left_triple", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_click(payload, Button::Left, 3).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("right_single", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_click(payload, Button::Right, 1).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("drag", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_drag(payload).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("hotkey", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_hotkey(payload).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("type", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_type(payload).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("scroll", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_scroll(payload).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on_with_ack("wait", move |payload, _, ack| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let result = session.on_wait(payload).await;
            session.result_wrapper(ack, result).await;
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on("finish", move |_, _| {
          let session = session.clone();
          async move {
            let mut session = session.lock_owned().await;
            let mut state = session.state.lock().await;
            state.status = AutomationStatus::Completed;
            session.on_state_update(state).unwrap();
            session.abandon().await.unwrap();
          }.boxed()
        });
    }
    {
      let session = session.clone();
      builder = builder
        .on("error", move |payload, _| {
          let session = session.clone();
          async move {
            let session = session.lock().await;
            let msg = from_payload::<types::ErrorEventData>(payload).unwrap();
            eprintln!("Get error from agent framework {msg:?}");
            let mut state = session.state.lock().await;
            state.status = AutomationStatus::Failed;
            state.error = Some(msg.message);
            session.on_state_update(state).unwrap();
          }.boxed()
        });
    }
    session.lock().await.socket = Some(builder.connect().await?);
    Ok(session)
  }

  async fn abandon(&mut self) -> Result<()> {
    self.abandoned = true;
    if let Some(socket) = self.socket.take() {
      socket.disconnect().await?;
    }
    Ok(())
  }

  pub async fn cancel(&mut self) -> Result<()> {
    let mut state = self.state.lock().await;
    state.status = AutomationStatus::Cancelled;
    self.on_state_update(state).unwrap();
    self.abandon().await
  }

  pub async fn get_state(&self) -> AutomationState {
    self.state.lock().await.clone()
  }

  async fn ack<D: Into<Payload>>(&self, ack_id: i32, data: D) -> Result<()> {
    if let Some(socket) = self.socket.as_ref() {
      socket.ack(ack_id, data).await?;
    }
    Ok(())
  }

  async fn emit<E: Into<Event>, D: Into<Payload>>(&self, event: E, data: D) -> Result<()> {
    if let Some(socket) = self.socket.as_ref() {
      socket.emit(event, data).await?;
    }
    Ok(())
  }

  fn on_state_update(&self, state: MutexGuard<'_, AutomationState>) -> Result<()> {
    if !self.abandoned {
      self
        .app
        .emit("stateUpdated", serde_json::to_value(state.clone())?)?
    }
    Ok(())
  }

  async fn on_automation_error(&self, err: &LuxDesktopError) {
    let mut state = self.state.lock().await;
    if !matches!(state.status, AutomationStatus::Cancelled) {
      state.status = AutomationStatus::Failed;
      state.error = Some(format!("{err:?}"));
      self.on_state_update(state).unwrap();
    }
  }

  async fn result_wrapper(&self, ack: i32, result: Result<()>) {
    let resp = match result {
      Ok(_) => json!({ "success": true }),
      Err(err) => {
        self.on_automation_error(&err).await;
        json!({ "error": format!("{err:?}") })
      }
    };
    if let Err(err) = self.ack(ack, resp).await {
      self.on_automation_error(&err).await;
    }
  }

  async fn on_open(&self, instruction: String, mode: String) -> Result<()> {
    let model: String = if mode == MODE_THINKER {
      MODEL_THINKER
    } else {
      MODEL_ACTOR
    }
    .into();
    let init = types::InitEventData {
      instruction,
      mode: Some(mode),
      model: Some(model),
      temperature: None,
    };
    self.emit("init", serde_json::to_value(&init)?).await?;
    Ok(())
  }

  async fn push_history(&self, action: Action) -> Result<()> {
    let mut state = self.state.lock().await;
    state.history.push(action);
    self.on_state_update(state)
  }

  async fn on_request_screenshot(&self, payload: Payload) -> Result<()> {
    let data = from_payload::<types::ScreenshotRequestData>(payload)?;
    // `xcap::Monitor` is not `Send` on windows, so dynamicly get monitor is needed here.
    let screenshot: DynamicImage = xcap::Monitor::all()?
      .into_iter()
      .find(|m| (m.x().unwrap() as f64 - self.x).powi(2) + (m.y().unwrap() as f64 - self.y).powi(2) < 1.0)
      .ok_or_else(LuxDesktopError::error_current_monitor)?
      .capture_image()?
      .into();
    self.push_history(Action::Screenshot {
      screenshot: {
        let mut buf: Vec<u8> = vec![];
        screenshot.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)?;
        base64::engine::general_purpose::STANDARD.encode(&buf)
      },
    }).await?;
    reqwest::Client::new()
      .put(data.presigned_url)
      .body({
        let mut buf: Vec<u8> = vec![];
        screenshot.resize_exact(1260, 700, FilterType::Lanczos3)
          .write_with_encoder(JpegEncoder::new_with_quality(
            &mut Cursor::new(&mut buf),
            95,
          ))?;
        buf
      })
      .send()
      .await?;
    Ok(())
  }

  fn get_coordinate(&self, x: usize, y: usize) -> (i32, i32) {
    (
      (x as f64 / 1000.0 * self.size_x + self.offset_x) as i32,
      (y as f64 / 1000.0 * self.size_y + self.offset_y) as i32,
    )
  }

  fn move_mouse(&self, enigo: &mut MutexGuard<enigo::Enigo>, x: usize, y: usize) -> Result<()> {
    let (x, y) = self.get_coordinate(x, y);
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(Into::into)
  }

  async fn on_click(
    &self,
    payload: Payload,
    button: Button,
    times: usize,
  ) -> Result<()> {
    let data = from_payload::<types::ClickEventData>(payload)?;
    self.push_history(Action::Click(data.clone())).await?;
    let mut enigo = self.enigo.lock().await;
    self.move_mouse(&mut enigo, data.x, data.y)?;
    sleep(Duration::from_secs(1)).await;
    for _ in 0..times {
      enigo.button(button, Direction::Click)?;
      sleep(Duration::from_millis(100)).await;
    }
    Ok(())
  }

  async fn on_drag(&self, payload: Payload) -> Result<()> {
    let data = from_payload::<types::DragEventData>(payload)?;
    self.push_history(Action::Drag(data.clone())).await?;
    let mut enigo = self.enigo.lock().await;
    self.move_mouse(&mut enigo, data.x1, data.y1)?;
    sleep(Duration::from_millis(500)).await;
    enigo.button(Button::Left, Direction::Press)?;
    sleep(Duration::from_millis(500)).await;
    self.move_mouse(&mut enigo, data.x2, data.y2)?;
    sleep(Duration::from_millis(500)).await;
    enigo.button(Button::Left, Direction::Release)?;
    Ok(())
  }

  async fn on_hotkey(&self, payload: Payload) -> Result<()> {
    let data = from_payload::<types::HotkeyEventData>(payload)?;
    self.push_history(Action::Hotkey(data.clone())).await?;
    let keys = data
      .combo
      .to_lowercase()
      .split("+")
      .map(|key| match key.trim() {
        #[cfg(target_os = "windows")]
        "accept" => Key::Accept,
        "add" => Key::Add,
        "alt" | "altleft" | "altright" => Key::Alt,
        #[cfg(target_os = "windows")]
        "apps" => Key::Apps,
        "backspace" => Key::Backspace,
        #[cfg(target_os = "windows")]
        "browserback" => Key::BrowserBack,
        #[cfg(target_os = "windows")]
        "browserfavorites" => Key::BrowserFavorites,
        #[cfg(target_os = "windows")]
        "browserforward" => Key::BrowserForward,
        #[cfg(target_os = "windows")]
        "browserhome" => Key::BrowserHome,
        #[cfg(target_os = "windows")]
        "browserrefresh" => Key::BrowserRefresh,
        #[cfg(target_os = "windows")]
        "browsersearch" => Key::BrowserSearch,
        #[cfg(target_os = "windows")]
        "browserstop" => Key::BrowserStop,
        "caps_lock" => Key::CapsLock,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "clear" => Key::Clear,
        "command" | "win" | "winleft" | "winright" => Key::Meta,
        #[cfg(target_os = "windows")]
        "convert" => Key::Convert,
        #[cfg(target_os = "macos")]
        "ctrl" | "ctrlleft" | "ctrlright" => Key::Meta,
        #[cfg(not(target_os = "macos"))]
        "ctrl" => Key::Control,
        #[cfg(not(target_os = "macos"))]
        "ctrlleft" => Key::LControl,
        #[cfg(not(target_os = "macos"))]
        "ctrlright" => Key::RControl,
        "decimal" => Key::Decimal,
        "del" | "delete" => Key::Delete,
        "divide" => Key::Divide,
        "down" => Key::DownArrow,
        "end" => Key::End,
        "enter" | "return" => Key::Return,
        "esc" | "escape" => Key::Escape,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "execute" => Key::Execute,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        "f13" => Key::F13,
        "f14" => Key::F14,
        "f15" => Key::F15,
        "f16" => Key::F16,
        "f17" => Key::F17,
        "f18" => Key::F18,
        "f19" => Key::F19,
        "f20" => Key::F20,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "f21" => Key::F21,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "f22" => Key::F22,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "f23" => Key::F23,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "f24" => Key::F24,
        #[cfg(target_os = "windows")]
        "final" => Key::Final,
        #[cfg(target_os = "macos")]
        "fn" => Key::Function,
        #[cfg(target_os = "windows")]
        "hangeul" => Key::Hangeul,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "hangul" => Key::Hangul,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "hanja" => Key::Hanja,
        "help" => Key::Help,
        "home" => Key::Home,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "insert" => Key::Insert,
        #[cfg(target_os = "windows")]
        "junja" => Key::Junja,
        #[cfg(target_os = "windows")]
        "kana" => Key::Kana,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "kanji" => Key::Kanji,
        #[cfg(target_os = "windows")]
        "launchapp1" => Key::LaunchApp1,
        #[cfg(target_os = "windows")]
        "launchapp2" => Key::LaunchApp2,
        #[cfg(target_os = "windows")]
        "launchmail" => Key::LaunchMail,
        #[cfg(target_os = "windows")]
        "launchmediaselect" => Key::LaunchMediaSelect,
        "left" => Key::LeftArrow,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "modechange" => Key::ModeChange,
        "multiply" => Key::Multiply,
        "nexttrack" => Key::MediaNextTrack,
        #[cfg(target_os = "windows")]
        "nonconvert" => Key::NonConvert,
        "num0" => Key::Numpad0,
        "num1" => Key::Numpad1,
        "num2" => Key::Numpad2,
        "num3" => Key::Numpad3,
        "num4" => Key::Numpad4,
        "num5" => Key::Numpad5,
        "num6" => Key::Numpad6,
        "num7" => Key::Numpad7,
        "num8" => Key::Numpad8,
        "num9" => Key::Numpad9,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "numlock" => Key::Numlock,
        "option" | "optionleft" => Key::Option,
        #[cfg(target_os = "macos")]
        "optionright" => Key::ROption,
        "pagedown" | "pgdn" => Key::PageDown,
        "pageup" | "pgup" => Key::PageUp,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "pause" => Key::Pause,
        "playpause" => Key::MediaPlayPause,
        "prevtrack" => Key::MediaPrevTrack,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "print" | "printscreen" | "prntscrn" | "prtsc" | "prtscr" => Key::PrintScr,
        "right" => Key::RightArrow,
        #[cfg(all(unix, not(target_os = "macos")))]
        "scrolllock" => Key::ScrollLock,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "select" => Key::Select,
        "shift" => Key::Shift,
        "shiftleft" => Key::LShift,
        "shiftright" => Key::RShift,
        #[cfg(target_os = "windows")]
        "sleep" => Key::Sleep,
        "space" => Key::Space,
        #[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
        "stop" => Key::MediaStop,
        "subtract" => Key::Subtract,
        "tab" => Key::Tab,
        "up" => Key::UpArrow,
        "volumedown" => Key::VolumeDown,
        "volumemute" => Key::VolumeMute,
        "volumeup" => Key::VolumeUp,
        // Missing: separator, yen
        key => Key::Unicode(*key.as_bytes().get(0).unwrap() as char),
      })
      .collect::<Vec<_>>();
    let mut enigo = self.enigo.lock().await;
    for _ in 0..data.count {
      for key in keys.iter() {
        enigo.key(*key, Direction::Press)?;
        sleep(Duration::from_millis(10)).await;
      }
      for key in keys.iter().rev() {
        enigo.key(*key, Direction::Release)?;
        sleep(Duration::from_millis(10)).await;
      }
      sleep(Duration::from_millis(100)).await;
    }
    Ok(())
  }

  async fn on_type(&self, payload: Payload) -> Result<()> {
    let data = from_payload::<types::TypeEventData>(payload)?;
    self.push_history(Action::Type(data.clone())).await?;
    let mut enigo = self.enigo.lock().await;
    enigo.text(&data.text)?;
    Ok(())
  }

  async fn on_scroll(&self, payload: Payload) -> Result<()> {
    let data = from_payload::<types::ScrollEventData>(payload)?;
    self.push_history(Action::Scroll(data.clone())).await?;
    let length = data.count as i32;
    let length = match data.direction {
      types::ScrollDirection::Down => length,
      types::ScrollDirection::Up => -length,
    };
    let mut enigo = self.enigo.lock().await;
    self.move_mouse(&mut enigo, data.x, data.y)?;
    enigo.scroll(length, Axis::Vertical)?;
    Ok(())
  }

  async fn on_wait(&self, payload: Payload) -> Result<()> {
    let data = from_payload::<types::WaitEventData>(payload).unwrap();
    self.push_history(Action::Wait(data.clone())).await?;
    sleep(Duration::from_millis(data.duration_ms as u64)).await;
    Ok(())
  }
}
