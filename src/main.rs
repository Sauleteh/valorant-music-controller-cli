#![allow(non_snake_case)]

mod constants;
mod session_container;
use constants::*;
use session_container::SessionContainer;
use std::path::Path;
use std::io::{stdin, stdout, BufRead, BufReader, Seek, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread::sleep;
use std::time::Duration;
use enigo::{Enigo, Key, Keyboard, Settings};
use notify::{Watcher, RecursiveMode, Result, RecommendedWatcher, Config};
use regex::Regex;
use windows_volume_control::AudioController;

static STATE: AtomicU8 = AtomicU8::new(States::NOT_IN_GAME);
fn getState() -> u8 { return STATE.load(Ordering::Relaxed); }
fn setState(newState: u8) { STATE.store(newState, Ordering::Relaxed); }

static mut SESSION_CONTAINER: Option<SessionContainer> = None;

fn playOrPauseMedia() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let _ = enigo.key(Key::MediaPlayPause, enigo::Direction::Click);
}

fn updateVolume(prevState: u8) {
    let volume = VOLUMES[getState() as usize];
    println!("Setting volume to: {}", volume);

    unsafe {
        if let Some(ref mut session_container) = SESSION_CONTAINER {
            let selectedSession = session_container.controller.get_session_by_name(session_container.sessionName.clone());
            selectedSession.unwrap().setVolume(volume);
            // TODO: Implementar un sistema de volumen más avanzado
        }
    }

    // Si el volumen objetivo es 0 o se viene de un estado con volumen 0, se pausa o reanuda la música.
    let prevVolume = VOLUMES[prevState as usize];
    if (volume == 0.0 && prevVolume > 0.0) || (prevVolume == 0.0 && volume > 0.0) {
        playOrPauseMedia();
    }
}

fn analyzeText(text: &str) -> u8 {
    let re = Regex::new(r"^\[(?P<date>[^\]]+)\]\[(?P<code>[^\]]+)\](?P<name>[^\:]+):\s*(?P<text>.+)$").unwrap();
    if let Some(captures) = re.captures(text) {
        // let date = &captures["date"];
        // let code = &captures["code"];
        let name = &captures["name"];
        let text = &captures["text"];

        // println!("Log date: {}", date);
        // println!("Log code: {}", code);
        // println!("Log name: {}", name);
        // println!("Text: {}", text);

        if name == "LogShooterGameState" {
            if text.contains("Match Ended") {
                println!("Match ended.");
                return States::NOT_IN_GAME;
            }
            else if text.contains("AShooterGameState::OnRoundEnded") {
                println!("Round ended.");
                return States::IN_GAME_PREPARING;
            }
            else if text.contains("Gameplay started at local time") && !text.contains("0.000000") {
                println!("Round started.");
                return States::IN_GAME_PLAYING;
            }
        }
        else if name == "LogGameFlowStateManager" {
            if text.contains("Reconcile called with state: TransitionToInGame and new state: InGame. Changing state") {
                println!("Match started.");
                return States::IN_GAME_PREPARING;
            }
        }
        else if name == "LogSkeletalMesh" {
            if text.contains("USkeletalMeshComponent::RecreateClothingActors") {
                println!("Player respawned.");
                return States::IN_GAME_PLAYING;
            }
        }
        else if name == "LogAresMinimapComponent" {
            if text.contains("Found Compute Position override") {
                println!("Player died.");
                return States::IN_GAME_DEAD;
            }
        }
    }
    
    return getState(); // No se ha producido ningún cambio, se mantiene el estado actual.
}

fn watchFile() -> Result<()> {
    let binding = std::env::var("LOCALAPPDATA").unwrap() + "\\VALORANT\\Saved\\Logs\\ShooterGame.log";
    // let binding = "D:\\Users\\Saulete\\Downloads\\test.txt";
    let path = Path::new(&binding);
    let mut f = std::fs::File::open(&path)?;
    let mut pos = std::fs::metadata(&path)?.len();

    let (tx, _) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    // No se puede comprobar actualizaciones en el archivo por culpa de cómo funciona Windows; por tanto,
    // se comprueba cada X segundos si el archivo ha sido modificado.
    loop {
        if std::fs::metadata(&path)?.len() != pos {
            f.seek(std::io::SeekFrom::Start(pos))?;
            pos = std::fs::metadata(&path)?.len();

            let reader = BufReader::new(&f);
            for line in reader.lines() {
                let text = line.unwrap();
                if !text.is_empty() {
                    let newState = analyzeText(&text);
                    if newState != getState() { // Si el estado ha cambiado, se actualiza.
                        let prevState = getState();
                        setState(newState);
                        updateVolume(prevState);
                    }
                }
            }
        }

        println!("Current state: {}", getState());
        sleep(Duration::from_secs(1));
    }
}

fn requestProcessSession() {
    unsafe {
        let mut audioController = AudioController::init(None);
        audioController.GetSessions();
        audioController.GetDefaultAudioEnpointVolumeControl();
        audioController.GetAllProcessSessions();
        let sessions = audioController.get_all_session_names();
        let selectedOption: u8;
        
        for (i, session) in sessions.iter().enumerate() {
            println!("{}. {}", i + 1, session);
        }
        println!();

        loop {
            let mut number = String::new();
            print!("Select a process to control the volume (Write a number): ");
            let _ = stdout().flush();
            std::io::stdin().read_line(&mut number).expect("Failed to read the option.");
            if let Ok(val) = number.trim().parse::<u8>() {
                if val > sessions.len() as u8 {
                    println!("The number must be between 0 and {}.", sessions.len());
                }
                else {
                    selectedOption = val;
                    break;
                }
            }
            else {
                println!("Please, enter a valid number.");
            }
        }

        let sessionName = sessions[(selectedOption - 1) as usize].to_string();
        let currentVolume = audioController.get_session_by_name(sessionName.clone()).unwrap().getVolume();

        SESSION_CONTAINER = Some(SessionContainer {
            controller: audioController,
            sessionName: sessionName,
            initialVolume: currentVolume
        })
    }
}

fn main() {
    requestProcessSession();

    ctrlc::set_handler(move || {
        println!("Exiting...");
        unsafe {
            if let Some(ref mut session_container) = SESSION_CONTAINER {
                let selectedSession = session_container.controller.get_session_by_name(session_container.sessionName.clone());
                selectedSession.unwrap().setVolume(session_container.initialVolume);
            }
        }
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    // TODO: Avisar de que el programa se está ejecutando y que se puede cerrar con Ctrl+C, NO cerrar el programa mediante la X.
    watchFile().unwrap();
}

/* TODO list:
 * - [X] Al actualizar el estado, se actualiza también el volumen (Implementar updateVolume)
 * - [ ] Implementar el sistema de volumen
 *     - [X] Suponemos que pausar y reanudar la música es sencillo: es una combinación de teclas (MediaPlayPause) y se hace al llegar/salir del volumen 0.
 *     - [X] Controlar el volumen se hará mediante el control de volumen del sistema
*      - [X] Se le da a elegir al usuario cual es la aplicación la cual se le va a controlar el volumen
 *     - [X] El volumen que se tenía antes del programa se recupera al cerrar el programa
 *     - [ ] Además, el volumen se hace de forma gradual, no instantánea
 * - [ ] Pasar la aplicación a EGUI
 */