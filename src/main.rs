use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Default)]
struct ShellContext {
    aliases: HashMap<String, String>,
    functions: HashMap<String, Vec<String>>,
    history_log: Vec<String>,
}

#[derive(PartialEq)]
enum ParserState {
    None,
    Aliases,
    Functions,
    Bash,
}

impl ShellContext {
    fn load_rc(&mut self, home: &str) {
        let rc_path = format!("{}/.walltancerc", home);
        if !Path::new(&rc_path).exists() {
            let default_rc = "aliases {\n  l=ls -la\n}\n\nbash {\n  echo \"Walltance loaded!\"\n}\n";
            let _ = fs::write(&rc_path, default_rc);
        }

        let Ok(content) = fs::read_to_string(&rc_path) else { return };

        let mut state = ParserState::None;
        let mut brace_depth = 0;
        let mut current_func_name = String::new();
        let mut bash_buffer = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }

            match state {
                ParserState::None => {
                    if trimmed == "aliases {" { state = ParserState::Aliases; }
                    else if trimmed == "functions {" { state = ParserState::Functions; }
                    else if trimmed == "bash {" { state = ParserState::Bash; }
                    else {
                        execute_line(trimmed, self, home, false);
                    }
                }

                ParserState::Aliases => {
                    if trimmed == "}" { state = ParserState::None; }
                    else if let Some((k, v)) = trimmed.split_once('=') {
                        self.aliases.insert(
                            k.trim().to_string(),
                            v.trim().trim_matches(|c| c == '"' || c == '\'').to_string()
                        );
                    }
                }

                ParserState::Functions => {
                    if trimmed == "}" && brace_depth == 0 { state = ParserState::None; }
                    else if trimmed.ends_with('{') {
                        current_func_name = trimmed.trim_end_matches('{').trim().to_string();
                        self.functions.insert(current_func_name.clone(), Vec::new());
                        brace_depth += 1;
                    } else if trimmed == "}" {
                        brace_depth -= 1;
                        if brace_depth == 0 { current_func_name.clear(); }
                    } else if !current_func_name.is_empty() {
                        if let Some(vec) = self.functions.get_mut(&current_func_name) {
                            vec.push(trimmed.to_string());
                        }
                    }
                }

                ParserState::Bash => {
                    if trimmed == "}" {
                        state = ParserState::None;
                        let _ = Command::new("bash")
                            .arg("-c")
                            .arg(&bash_buffer)
                            .spawn()
                            .and_then(|mut child| child.wait());
                        bash_buffer.clear();
                    } else {
                        bash_buffer.push_str(line);
                        bash_buffer.push('\n');
                    }
                }
            }
        }
    }
}

fn expand_env(text: &str) -> String {
    let mut expanded = text.to_string();
    for (key, value) in env::vars() {
        let target = format!("${}", key);
        if expanded.contains(&target) {
            expanded = expanded.replace(&target, &value);
        }
    }
    expanded
}

fn execute_line(input: &str, ctx: &mut ShellContext, home_dir: &str, record: bool) -> bool {
    let mut input = expand_env(input.trim());
    if input.is_empty() { return true; }

    if record { ctx.history_log.push(input.clone()); }
    if input == "exit" { return false; }

    // --- 1. ПРОВЕРКА АЛИАСОВ (До всего остального) ---
    let first_word = input.split_whitespace().next().unwrap_or("");
    if let Some(alias_val) = ctx.aliases.get(first_word) {
        // Заменяем только первое слово, чтобы не сломать аргументы
        input = input.replacen(first_word, alias_val, 1);
    }

    // Пересчитываем первое слово после возможной замены алиаса
    let first_word = input.split_whitespace().next().unwrap_or("");

    // --- 2. ВСТРОЕННЫЕ КОМАНДЫ ---
    if input == "history" {
        for (i, cmd) in ctx.history_log.iter().enumerate() {
            println!("{:>4}  {}", i + 1, cmd);
        }
        return true;
    }

    if first_word == "cd" {
        let target = input.strip_prefix("cd ").unwrap_or("~").trim();
        let dest = if target == "~" || target.is_empty() { home_dir.to_string() } else { target.to_string() };
        if let Err(_) = env::set_current_dir(&dest) {
            eprintln!("walltance: no such file or directory: {}", dest);
        }
        return true;
    }

    // --- 3. ФУНКЦИИ ---
    if let Some(commands) = ctx.functions.get(first_word).cloned() {
        for cmd in commands {
            execute_line(&cmd, ctx, home_dir, false);
        }
        return true;
    }

    // --- 4. СИСТЕМНЫЕ КОМАНДЫ (Пайпы и &&) ---
    let and_chunks: Vec<&str> = input.split("&&").collect();

    for chunk in and_chunks {
        let chunk = chunk.trim();
        if chunk.is_empty() { continue; }

        let commands: Vec<&str> = chunk.split('|').collect();
        let mut children = Vec::new();
        let mut previous_stdout = None;
        let mut chunk_success = true;

        for (i, cmd) in commands.iter().enumerate() {
            let cmd_trimmed = cmd.trim();
            let parts: Vec<&str> = cmd_trimmed.split_whitespace().collect();
            if parts.is_empty() { continue; }

            let stdin = previous_stdout.map_or(Stdio::inherit(), |stdout| Stdio::from(stdout));
            let stdout = if i < commands.len() - 1 { Stdio::piped() } else { Stdio::inherit() };

            match Command::new(parts[0]).args(&parts[1..]).stdin(stdin).stdout(stdout).spawn() {
                Ok(mut child) => {
                    previous_stdout = child.stdout.take();
                    children.push(child);
                }
                Err(_) => {
                    eprintln!("walltance: command not found: {}", parts[0]);
                    chunk_success = false;
                    break;
                }
            }
        }

        for mut child in children {
            let _ = child.wait();
        }

        if !chunk_success { break; }
    }

    true
}

fn main() {
    let mut rl = DefaultEditor::new().expect("FAIL: Rustyline");
    let home_dir = env::var("HOME").unwrap_or_default();
    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());

    let mut ctx = ShellContext::default();
    ctx.load_rc(&home_dir);

    loop {
        let cwd = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let mut cwd_str = cwd.to_string_lossy().to_string();
        if !home_dir.is_empty() && cwd_str.starts_with(&home_dir) {
            cwd_str = cwd_str.replacen(&home_dir, "~", 1);
        }

        let prompt = format!(
            "\x1b[1;32m{}\x1b[0m \x1b[1;34m{}\x1b[0m \x1b[1;35m%#\x1b[0m ",
            user, cwd_str
        );

        match rl.readline(&prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                if !execute_line(&line, &mut ctx, &home_dir, true) { break; }
            },
            Err(ReadlineError::Interrupted) => { println!("^C"); },
            Err(ReadlineError::Eof) => break,
            Err(err) => { eprintln!("Error: {:?}", err); break; }
        }
    }
}
