use clap::{Parser, ValueEnum};
use ksway::{ipc_command, Client};
use serde_json::{from_str, Value};
use std::env::var;
use std::usize;

/// Simple command to switch workspaces with optional output awareness for Sway/i3
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sway/i3 socket path
    #[arg(short, long, default_value_t = var("SWAYSOCK").unwrap())]
    sock: String,

    /// Action
    #[arg(value_enum)]
    action: Action,

    /// Move to new workspace
    #[arg(short, long = "move", default_value_t = false)]
    move_ws: bool,

    /// Do not focus to new workspace
    #[arg(short, long = "no-focus", default_value_t = false)]
    no_focus_ws: bool,

    /// Print workspace number to stdout
    #[arg(short = 'o', long = "stdout", default_value_t = false)]
    stdout_ws: bool,
}

#[derive(ValueEnum, Clone)]
enum Action {
    NextOnOutput,
    PrevOnOutput,
}

fn get_workspaces(client: &mut Client) -> Vec<Value> {
    return from_str(&String::from_utf8_lossy(
        &client.ipc(ipc_command::get_workspaces()).unwrap(),
    ))
    .unwrap();
}

fn focus_ws_named(client: &mut Client, name: String) -> Result<Vec<u8>, ksway::Error> {
    return client.ipc(ipc_command::run(format!("workspace {name}")));
}

fn move_ws_named(client: &mut Client, name: String) -> Result<Vec<u8>, ksway::Error> {
    return client.ipc(ipc_command::run(format!("move workspace {name}")));
}

fn find_on_output(
    workspaces: &Vec<Value>,
    current_name: String,
    step: i32,
    output: String,
) -> String {
    let output_wss: Vec<&Value> = workspaces
        .into_iter()
        .filter(|w| w["output"].to_string() == output)
        .collect();

    let wss_names: Vec<String> = output_wss
        .into_iter()
        .map(|w| w["name"].to_string())
        .collect();

    let n_ws_output: i32 = wss_names.len() as i32;

    let match_num: usize = wss_names
        .clone()
        .into_iter()
        .position(|w| w.to_string() == current_name)
        .unwrap();

    //Fails on negatives!
    // let next: usize = ((match_num as i32 + step) % n_ws_output) as usize;
    let mut next: i32 = (match_num as i32 + step) % n_ws_output;
    if next < 0 {
        next += n_ws_output
    }

    let next_name: String = wss_names[next as usize].to_string();
    return next_name;
}

fn main() {
    //println!("My sway-workspace");
    let args: Args = Args::parse();

    let mut client = Client::connect_to_path(args.sock.to_owned()).unwrap();

    let workspaces: &Vec<Value> = &get_workspaces(&mut client);

    let current_ws: &Value = workspaces
        .into_iter()
        .filter(|w| w["focused"] == true)
        .nth(0)
        .unwrap();

    let current_ws_name: String = current_ws["name"].to_string();
    let current_output: String = current_ws["output"].to_string();

    let step: i32 = match args.action {
        Action::NextOnOutput => 1,
        Action::PrevOnOutput => -1,
    };

    let name: String = find_on_output(&workspaces, current_ws_name, step, current_output);

    if args.move_ws {
        move_ws_named(&mut client, name.clone()).unwrap();
    }

    if !args.no_focus_ws {
        focus_ws_named(&mut client, name.clone()).unwrap();
    }

    if args.stdout_ws {
        print!("{}", name);
    }
}
