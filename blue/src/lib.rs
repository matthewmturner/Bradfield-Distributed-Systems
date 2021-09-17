use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::net::TcpStream;
use std::path::Path;
use std::str::FromStr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use serde_json::json;

#[derive(Debug)]
pub enum Command {
    Get,
    Set,
}

impl FromStr for Command {
    type Err = ();
    fn from_str(input: &str) -> Result<Command, Self::Err> {
        match input {
            "get" | "Get" | "GET" => Ok(Command::Get),
            "set" | "Set" | "SET " => Ok(Command::Set),
            _ => Err(()),
        }
    }
}
pub struct UserInput {
    pub text: String,
    pub is_valid: bool,
    pub command: Option<Command>,
    pub value: Option<String>,
}

impl UserInput {
    pub fn new(input: String) -> UserInput {
        let (validity, command, value) = UserInput::validate(&input);
        match validity {
            true => UserInput {
                text: input,
                is_valid: true,
                command,
                value,
            },
            false => UserInput {
                text: input,
                is_valid: false,
                command: None,
                value: None,
            },
        }
    }
    fn validate(input: &str) -> (bool, Option<Command>, Option<String>) {
        let commands = vec!["get", "set"];
        let tokens: Vec<&str> = input.split(' ').collect();

        let validity = match tokens.len() {
            0 => (false, None, None),
            1 => {
                let command = tokens[0].trim();
                if command == "get" {
                    (true, Some(Command::Get), None)
                } else {
                    (false, None, None)
                }
            }
            2 => {
                let command = tokens[0].trim();
                if commands.iter().any(|&c| c == command) {
                    (
                        true,
                        Some(Command::from_str(&command).unwrap()),
                        Some(tokens[1].trim().to_string()),
                    )
                } else {
                    (false, None, None)
                }
            }
            _ => (false, None, None),
        };
        validity
    }
}

fn parse_stream_input(stream: &mut TcpStream, input_num: &mut i32) -> io::Result<UserInput> {
    let msg = format!("[{}] ", input_num);
    stream.write(msg.as_bytes())?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line)?;
    println!("Read {} bytes", bytes_read);

    let user_input = UserInput::new(line);
    if !user_input.is_valid {
        println!("Invalid input")
    }
    Ok(user_input)
}

fn get_stream_input(
    stream: &mut TcpStream,
    input: UserInput,
    store: &mut HashMap<String, String>,
) -> io::Result<()> {
    // TODO: Get from store or file
    if let Some(key) = input.value {
        println!("Getting key={}", key);
        let value = store.get(&key);
        if let Some(v) = value {
            let msg = format!("{}\n", v);
            stream.write(msg.as_bytes())?;
        };
    } else {
        println!("Getting store");
        let msg = format!("{}\n", json!(store).to_string());
        stream.write(msg.as_bytes())?;
    }
    Ok(())
}

fn store_stream_input(
    stream: &mut TcpStream,
    input: UserInput,
    store: &mut HashMap<String, String>,
) -> io::Result<()> {
    let set_value = input.value.expect("No set value provided");

    let tokens: Vec<&str> = set_value.split("=").collect();

    if tokens.len() == 2 {
        let key = tokens[0];
        let value = tokens[1].trim();
        store.insert(key.to_string(), value.to_string());
        println!("Storing {}={}", key, value);
        let msg = format!("Succesfully wrote key={}\n", key);
        stream.write(msg.as_bytes())?;
        return Ok(());
    }
    Err(io::Error::new(
        ErrorKind::InvalidData,
        "Set commands must be of format key=val",
    ))
}

fn persist_store(store: &mut HashMap<String, String>, path: &Path) -> io::Result<()> {
    let json = json!(store);
    fs::write(path, json.to_string())?;
    Ok(())
}

pub fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    println!("Connection established with: {}", stream.peer_addr()?);
    let mut input_num: i32 = 1;
    let welcome = "Welcome to Blue!\n";
    stream.write(welcome.as_bytes())?;

    let store_path = Path::new("data.json");

    let mut store: HashMap<String, String> = match store_path.exists() {
        true => {
            let existing_store = fs::read_to_string(&store_path)?;
            serde_json::from_str(&existing_store)?
        }
        false => HashMap::new(),
    };

    loop {
        let input = parse_stream_input(&mut stream, &mut input_num)?;
        match input.command {
            Some(Command::Get) => get_stream_input(&mut stream, input, &mut store)?,
            Some(Command::Set) => {
                store_stream_input(&mut stream, input, &mut store)?;
                persist_store(&mut store, store_path)?;
            }
            None => println!("Figure this out"),
        }
        input_num += 1;
    }
}

pub fn handle_stream(
    mut stream: TcpStream,
    store: Arc<Mutex<HashMap<String, String>>>,
    store_path: Arc<&Path>,
) -> io::Result<()> {
    println!("Connection established with: {}", stream.peer_addr()?);
    let mut input_num: i32 = 1;
    let welcome = "Welcome to Blue!\n";
    stream.write(welcome.as_bytes())?;

    loop {
        let input = parse_stream_input(&mut stream, &mut input_num)?;
        let mut store = store.lock().unwrap();

        match input.command {
            Some(Command::Get) => get_stream_input(&mut stream, input, &mut store)?,
            Some(Command::Set) => {
                store_stream_input(&mut stream, input, &mut store)?;
                persist_store(&mut store, &store_path)?;
            }
            None => println!("Figure this out"),
        }
        input_num += 1;
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing", id);
            job();
        });
        Worker { id, thread }
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}
