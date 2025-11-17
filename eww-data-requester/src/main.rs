use enums::Requests;
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

const REQUEST_SOCKET_PATH: &str = "/tmp/eww_data/requests.socket";

fn help() {
    eprintln!("Usage: <command> [args...]");
    eprintln!("Commands:");
    eprintln!("  listen <socket_name> - Listen on a Unix socket and print incoming lines.");
    eprintln!("  send <request_type>  - Send a request enum to the data provider.");
    eprintln!("Available request types: notifications");
    std::process::exit(1);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        help();
    }

    let command = &args[1];

    match command.as_str() {
        "listen" => {
            if args.len() < 3 {
                help();
            }
            let socket_name = &args[2];
            let dir_path = "/tmp/eww_data";
            let socket_path = format!("{}/{}.socket", dir_path, socket_name);
            let path = Path::new(&socket_path);

            fs::create_dir_all(dir_path)?;

            if path.exists() {
                // eprintln!("Removing existing socket: {}", socket_path);
                fs::remove_file(path)?;
            }

            let listener = UnixListener::bind(path)?;
            // eprintln!("Listening on socket: {}", socket_path);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let mut reader = BufReader::new(stream);
                        let mut line = String::new();
                        while reader.read_line(&mut line)? > 0 {
                            let mut line_copy = line.clone();
                            line_copy = line_copy.trim().to_string();
                            if !line_copy.is_empty() {
                                println!("{}", line_copy);
                            }
                            line.clear();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }
            }
        }
        "send" => {
            if args.len() < 3 {
                help();
            }
            let request_type = &args[2];
            let request = match request_type.as_str() {
                "notifications" => Requests::Notifications,
                "virtualkeyboard" => Requests::VirtualKeyboard,
                "settingsmenu" => Requests::SettingsMenu,
                _ => {
                    eprintln!("Unknown request type: {}", request_type);
                    std::process::exit(1);
                }
            };

            if let Ok(data) = postcard::to_allocvec(&request) {
                let mut stream = UnixStream::connect(REQUEST_SOCKET_PATH)?;
                stream.write_all(&data)?;
                //eprintln!("Sent request: {:?}", request);
            } else {
                eprintln!("Failed to postcard");
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            help();
        }
    }

    Ok(())
}
