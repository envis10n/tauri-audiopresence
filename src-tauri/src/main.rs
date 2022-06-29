#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayEvent};
use audiopresence::{MediaManager, MediaProps, OsMediaProps, PlayerStatus};
use std::{sync::{Arc, Mutex}, fmt::Display};
use crossbeam::atomic::AtomicCell;
use discord_rich_presence::{ DiscordIpc, DiscordIpcClient, activity };
use tokio::time;

const APP_ID: &'static str = "990123446186958918";

fn set_discord_presence(client: Arc<Mutex<DiscordIpcClient>>, props: MediaProps) -> Result<(), Box<dyn std::error::Error>> {
  let title: String = {
    if cfg!(target_os = "windows") {
      if !props.album_artist.is_empty() {
        format!("{} - {}", props.title, props.album_artist)
      } else {
        props.title.clone()
      }
    } else {
      if !props.album_title.is_empty() {
        format!("{} - {}", props.title, props.album_title)
      } else {
        props.title.clone()
      }
    }
  };
  let mut client = client.lock().unwrap();
  (*client).set_activity(activity::Activity::new()
    .details(props.artist.as_str())
    .state(title.as_str())
  )
}

fn clear_discord_presence(client: Arc<Mutex<DiscordIpcClient>>) -> Result<(), Box<dyn std::error::Error>> {
  let mut client = client.lock().unwrap();
  (*client).set_activity(activity::Activity::new())
}

fn debugprint(msg: impl Display) {
  if cfg!(debug_assertions) {
    println!("{}", msg);
  }
}

fn main() {
  let context = tauri::generate_context!();

  let quit = CustomMenuItem::new("quit".to_string(), "Quit");

  let tray_menu = SystemTrayMenu::new()
    .add_item(quit);

  let tray = SystemTray::new()
    .with_menu(tray_menu);

  tauri::Builder::default()
    .system_tray(tray)
    .on_system_tray_event(|_app, event| match event {
      SystemTrayEvent::MenuItemClick { id, .. } => {
        match id.as_str() {
          "quit" => {
            std::process::exit(0);
          }
          _ => {}
        }
      }
      _ => {}
    })
    .setup(|_app| {
      let client = Arc::new(Mutex::new(DiscordIpcClient::new(APP_ID).unwrap()));
      {
        let mut client = client.lock().unwrap();
        (*client).connect().unwrap();
        debugprint("Discord client connected.");
      }

      debugprint("MediaManager created.");

      let start_props = match MediaManager::currently_playing() {
        Ok(props) => {
          set_discord_presence(client.clone(), props.clone()).unwrap();
          debugprint(format!("Starting properties: {:?}", props));
          props
        }
        Err(_) => MediaProps::new(),
      };

      let last_state = Arc::new(AtomicCell::new(start_props));

      let player_status = Arc::new(AtomicCell::new(PlayerStatus::None));

      tauri::async_runtime::spawn(async move {
        loop {
          time::sleep(time::Duration::from_millis(2000)).await;
          let last_state = last_state.clone();
          let player_status = player_status.clone();
          let status_ = player_status.take();
          let client = client.clone();
          match MediaManager::player_status() {
            Ok(pstatus) => {
              match pstatus {
                PlayerStatus::None => {
                  if status_ != PlayerStatus::None {
                    debugprint("Player stopped.");
                    clear_discord_presence(client.clone()).unwrap();
                    last_state.take();
                  }
                },
                PlayerStatus::Paused(_) => {
                  if status_ != pstatus {
                    debugprint("Player paused.");
                    clear_discord_presence(client).unwrap();
                    last_state.take();
                  }
                }
                PlayerStatus::Playing(_) => {
                  if status_ != pstatus {
                    debugprint("Player unpaused.");
                  }
                  match MediaManager::currently_playing() {
                    Ok(state) => {
                      let last_ = last_state.take();
                      if last_ != state {
                        debugprint(format!("Updated properties {:?}", state));
                        last_state.store(state.clone());
                        set_discord_presence(client, state).unwrap();
                      } else {
                        last_state.store(last_);
                      }
                    }
                    Err(e) => {
                      if e.to_string().as_str() != "ERROR: The operation completed successfully." { // temp to not panic when we just can't get a session.
                        println!("ERROR: {}", e);
                        break;
                      } else {
                        clear_discord_presence(client).unwrap();
                        last_state.take();
                      }
                    }
                  }
                }
              }
              player_status.store(pstatus);
            }
            Err(e) => {
              if e.to_string().as_str() != "ERROR: The operation completed successfully." { // temp to not panic when we just can't get a session.
                println!("ERROR: {}", e);
                break;
              } else {
                clear_discord_presence(client).unwrap();
              }
            }
          }
        }
        std::process::exit(1);
      });

      debugprint("Manager set to update on changes.");

      Ok(())
    })
    .run(context)
    .expect("error while running tauri application");
}
