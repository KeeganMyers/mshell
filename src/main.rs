use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::process::Command;
use std::process::Child;
use std::ffi::OsStr;
use std::path::Path;
use std::env;
use std::io::Error;
use std::iter::Peekable;
use std::process::Stdio;
use std::str::SplitWhitespace;

fn cd(args: SplitWhitespace) -> Option<Child>
{
    let new_dir = args.peekable().peek().map_or("/", |x| *x);
    let root = Path::new(new_dir);
    if let Err(e) = env::set_current_dir(&root) {
    eprintln!("{}", e);
    }
    None
}

fn spawn_command<I,S,P>(command: &str, args: I,commands: &mut Peekable<P> , previous_command: Option<Child>) -> Result<Child, Error>
    where I: IntoIterator<Item = S>,
          S: AsRef<OsStr>,
          P: Iterator
{
    let stdin = previous_command
            .map_or(Stdio::inherit(), |output: Child| Stdio::from(output.stdout.unwrap())
            );

    let stdout = if commands.peek().is_some() {
    Stdio::piped()
    } else {
    Stdio::inherit()
    };

    Command::new(command)
        .args(args)
        .stdin(stdin)
        .stdout(stdout)
        .spawn()
}

fn main() {
  loop {
    print!("> ");
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    let mut commands = input.trim().split(" | ").peekable();
    let mut previous_command = None;

    while let Some(command) = commands.next() {
        
        let mut parts = command.trim().split_whitespace();
        let command = parts.next().unwrap();
        let args = parts;

        match command {
            "cd" =>  { previous_command = cd(args)},
            "exit" => return,
            command => {
                            match spawn_command(command, args, &mut commands, previous_command) {
                                Ok(output) => {previous_command = Some(output); },
                                Err(e) => {
                                            previous_command = None;
                                            eprintln!("{}", e)
                                        },
                            }
                        }
        }
    }
        if let Some(mut final_command) = previous_command {
            final_command.wait();
        }
  }
}
