extern crate rustyline;
use std::io::{stdin,stdout,Write, Error};
use std::process::{Command, Child};
use std::ffi::OsStr;
use std::path::Path;
use std::env;
use std::iter::Peekable;
use std::process::Stdio;
use std::str::SplitWhitespace;
use env_logger;
use std::borrow::Cow::{self, Borrowed, Owned};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, Helper, KeyPress};

struct ShellHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for ShellHelper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for ShellHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Helper for ShellHelper {}


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
    env_logger::init();
    let mut rl = Editor::<()>::new();
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();
    let h = ShellHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_owned(),
    };
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyPress::Meta('N'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyPress::Meta('P'), Cmd::HistorySearchBackward);

  loop {
    let p = format!("> ");
    rl.helper_mut().unwrap().colored_prompt = format!("\x1b[1;32m{}\x1b[0m", p);
    let line_in = match rl.readline(&p) {
                    Ok(line) =>  {
                                 rl.add_history_entry(line.as_str());
                                 line
                                 },
                    Err(ReadlineError::Interrupted) => {
                            break
                        },
                        Err(ReadlineError::Eof) => {
                            break
                        },
                        Err(err) => {
                            println!("Error: {:?}", err);
                            break
                        }
                    Err(_) => "".to_string(),

    };
    let mut commands = line_in.trim().split(" | ").peekable();
    let mut previous_command = None;

    while let Some(command) = commands.next() {
        let mut parts = command.trim().split_whitespace();
        let command = parts.next().unwrap();
        let args = parts;

        match command {
            "cd" =>  { previous_command = cd(args)},
            "exit" => {
                       rl.save_history("history.txt").unwrap();
                       return
                       },
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
  rl.save_history("history.txt").unwrap();
}
