use hyprland::data::{Client, Workspace, Workspaces};
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;

fn main() {
    // turns the args to a vector
    let args: Vec<String> = env::args().collect();
    let mut args = args.into_iter();
    // skips the binary name
    args.next();
    // parses the command
    let arg = args.next().unwrap_or_default();
    // turns the command into a string literal
    let arg = arg.as_str();
    // sets up the event listener
    let mut el = EventListener::new();
    // sentinal for the event listener
    let mut listen = false;
    // match the arg
    match arg {
        "work" => {
            // the current workspace
            work(&mut el);
            listen = true;
        }

        "window" => {
            // the current window class
            window(&mut el);
            listen = true;
        }
        "bat" => bat(),
        "open_windows" => {
            // Adds event listeners for everything relating to windows and workspaces
            el.add_workspace_added_handler(|_, _| open_windows());
            el.add_workspace_change_handler(|_, _| open_windows());
            el.add_workspace_destroy_handler(|_, _| open_windows());
            el.add_window_open_handler(|_, _| open_windows());
            el.add_window_close_handler(|_, _| open_windows());
            el.add_window_moved_handler(|_, _| open_windows());
            el.add_window_title_change_handler(|_, _| open_windows());
            el.add_active_monitor_change_handler(|_, _| open_windows());
            el.add_active_window_change_handler(|_, _| open_windows());
            // prints the information for the initial usage
            open_windows();
            listen = true
        }
        "mem" => mem(),
        _ => {}
    }
    if listen {
        el.start_listener().unwrap();
    }
}

fn work(el: &mut EventListener) {
    // prints out for first launch
    println!("{}", Workspace::get_active().unwrap().id);
    // adds the event listener
    el.add_workspace_change_handler(|data, _| println!("{}", data));
}

fn window(el: &mut EventListener) {
    // add event handler for when a window is opened / changed to get the curretn window name
    el.add_active_window_change_handler(|data, _| {
        println!("{}", data.unwrap().window_class);
    });
    // event handler for when a window is closed on a workspace with no other windows
    el.add_window_close_handler(|_, _| {
        let active = Client::get_active();
        // returns if error
        let f = match active {
            Ok(t) => t,
            Err(_) => return,
        };
        if f.is_none() {
            println!("Desktop")
        };
    });
}

// struct that gives only the information I need
#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceWindow {
    id: i32,
    windows: i32,
}

// gets the workspaces with applications open
fn open_windows() {
    // empty vector
    let mut workspaces = vec![];
    // 1 - 10 range to add workspaces 1 - 10
    for i in 1..=10 {
        workspaces.push(WorkspaceWindow { id: i, windows: 0 })
    }
    // active workspaces
    let client = Workspaces::get();
    // checks if it is safe to unwrap client
    if client.is_ok() {
        // unwraps the client
        let clients = client.unwrap();
        // iterates over every client
        for client in clients {
            // iterates over every workspace
            for workspace in &mut workspaces {
                // the client lines up with the workspace id
                if client.id == workspace.id {
                    // is so set the amount of windows in workspace to the amount in client
                    workspace.windows = client.windows as i32;
                }
            }
        }
        // turn it to json
        let json = serde_json::to_string(&workspaces);
        // print the json
        println!("{}", json.unwrap());
    }
}

fn bat() {
    // sets the command to bat and the argument to the battery copacity
    let command_out = Command::new("/bin/bat")
        .arg("/sys/class/power_supply/BAT0/capacity")
        .output();
    // if command fail bail out
    if command_out.is_err() {
        return;
    }
    let command_out = command_out.unwrap();
    // converts the command output to an i32
    let bat_percent = String::from_utf8_lossy(&command_out.stdout)
        .to_string()
        .trim()
        .parse()
        .unwrap();
    // maps the i32 to css classes
    match bat_percent {
        0..=25 => {
            println!("low")
        }
        26..=50 => {
            println!("warn")
        }
        51..=75 => {
            println!("alright")
        }
        76..=100 => {
            println!("good")
        }
        _ => {}
    }
}

const DIGIT_BITSHIFT: usize = 10;

// provides the free column value of output as the free -m command
fn mem() {
    // read the same file the free command uses
    let command_out = Command::new("/bin/bat").arg("/proc/meminfo").output();
    // if command fail bail out
    if command_out.is_err() {
        return;
    }
    let command_out = command_out.unwrap();
    // turn command to string
    let command_out = String::from_utf8_lossy(&command_out.stdout);
    // initize varibles
    let mut mem_total = 0;
    let mut mem_free = 0;
    // split into lines
    for line in command_out.lines() {
        // check if the line starts with MemTotal
        if line.starts_with("MemTotal:") {
            mem_total = line
                // turn the line into chars
                .chars()
                // filter out anything that is not a digit
                .filter_map(|char| char.to_digit(DIGIT_BITSHIFT as u32))
                // turn into a vector of u32
                .collect::<Vec<u32>>()
                .iter()
                // turn into a string
                .map(|digit| digit.to_string())
                .collect::<String>()
                // turn that string into a int
                .parse::<i32>()
                .unwrap()
                // bitshift from kibibytes to mibibytes
                >> DIGIT_BITSHIFT

        // check if the line starts with MemTotal
        } else if line.starts_with("MemAvailable:") {
            mem_free = line
                // follow the same steps as MemTotal
                .chars()
                .filter_map(|char| char.to_digit(DIGIT_BITSHIFT as u32))
                .collect::<Vec<u32>>()
                .iter()
                .map(|digit| digit.to_string())
                .collect::<String>()
                .parse::<i32>()
                .unwrap()
                >> DIGIT_BITSHIFT
        }
    }
    // provide the output
    println!("{}", mem_total - mem_free)
}
