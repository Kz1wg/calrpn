mod calcrpn;
use calcrpn::{manage_stack, CalcNum, DegMode, Memorize};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use rustyline::{config::Configurer, DefaultEditor};
use std::{
    collections::{BTreeMap, VecDeque},
    io,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut stack: VecDeque<CalcNum> = VecDeque::new();
    let mut memo_map: BTreeMap<String, CalcNum> = BTreeMap::new();
    let mut degmode = DegMode::Deg;
    let mut memo_mode: Option<Memorize> = None;
    let mut result = String::new();
    let mut memory = String::new();
    let mut message = String::new();
    let mut decimal_point: usize = 3;
    let mut last_stackresult = (stack.clone(), result.clone());
    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };

    let mut input = String::new();
    let mut readline = DefaultEditor::new()?;
    let mut input_log: VecDeque<String> = VecDeque::new();
    readline.set_max_history_size(20)?;

    loop {
        let result_len = calcrpn::STACK_SIZE.min(stack.len()) + 2;
        if last_stackresult.0.len() > stack.len() + 1 {
            terminal.clear()?;
        };
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(4),
                        Constraint::Length(result_len as u16),
                        Constraint::Length(2),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            // ステータスバー
            let status_block = Block::default().borders(Borders::NONE);
            let status_text = Paragraph::new(format!(
                "Fix:{} | {:?} | MemoMode:{:?}",
                decimal_point, degmode, memo_mode
            ))
            .block(status_block);
            f.render_widget(status_text, chunks[0]);

            //Memory
            let memory_block = Block::default().borders(Borders::NONE);
            let memory_text = Paragraph::new(memory.as_ref()).block(memory_block);
            f.render_widget(memory_text, chunks[1]);
            // Helper メッセージ
            let help_block = Block::default().title("Message").borders(Borders::ALL);
            let help_text = Paragraph::new(format!(
                "Enter: Calc | quit or q : Quit | Undo : undo{}{}",
                sepalator, message
            ))
            .block(help_block);
            f.render_widget(help_text, chunks[2]);

            //結果表示
            let result_block = Block::default().title("Result Stack").borders(Borders::ALL);
            let result_text = Paragraph::new(result.as_ref()).block(result_block);
            f.render_widget(result_text, chunks[3]);

            // 入力
            let input_block = Block::default().title("Input").borders(Borders::TOP);
            let input_text = Paragraph::new(input.as_ref()).block(input_block);
            f.render_widget(input_text, chunks[4]);
            // 余白
            let status_block = Block::default().borders(Borders::TOP);
            let status_text = Paragraph::new(" ".to_string()).block(status_block);
            f.render_widget(status_text, chunks[5]);
        })?;

        // 入力のカーソルを表示
        let input_col = 1;
        let input_row = result_len as u16 + 8;
        let cursor_col = input_col + input.len() as u16;

        execute!(
            terminal.backend_mut(),
            crossterm::cursor::MoveTo(cursor_col, input_row),
            crossterm::cursor::Show
        )?;

        input = readline.readline(" >> ")?;
        input_log.push_back(input.clone());

        match input.trim() {
            "undo" => {
                // undoの処理
                stack = last_stackresult.0.clone();
                result = last_stackresult.1.clone();
                message = "Undo".to_string();
                continue;
            }
            "quit" | "q" => {
                break;
            }
            _ => {
                let app_command = input.split_whitespace().collect::<Vec<&str>>();
                if app_command.len() == 2 {
                    match app_command[0] {
                        "fix" => {
                            // fixの処理
                            let fix = app_command[1].parse::<usize>().unwrap_or(3);
                            decimal_point = fix;
                        }
                        "clv" => {
                            memo_map.remove_entry(app_command[1]);
                            update_log(&mut input_log, &mut message);
                            update_memo(&memo_map, &mut memory);
                        }
                        _ => (),
                    }
                }

                let temp_stack = stack.clone();
                let temp_result = result.clone();

                match manage_stack(
                    &input,
                    &mut stack,
                    &mut degmode,
                    &mut memo_map,
                    &mut memo_mode,
                ) {
                    Ok(()) => {
                        result.clear();
                        memory.clear();
                        // 入力を履歴に追加
                        readline.add_history_entry(input.clone())?;
                        // スタックを更新
                        update_stack(&stack, &mut result, decimal_point);
                        update_memo(&memo_map, &mut memory);
                        // undo用スタックに保持
                        last_stackresult = (temp_stack, temp_result);
                        update_log(&mut input_log, &mut message);
                    }
                    Err(e) => {
                        message = format!("Error: {}", e);
                        input_log.pop_back();
                    }
                }
            }
        }
    }

    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

fn update_log(input_log: &mut VecDeque<String>, message: &mut String) {
    if input_log.len() > 8 {
        input_log.pop_front();
    }
    *message = input_log
        .iter()
        .rev()
        .fold("Hist: ".to_string(), |acc, x| acc + x + " ← ");
}

fn update_memo(memo_map: &BTreeMap<String, CalcNum>, memory: &mut String) {
    // memo_mapをStringに変換しスペースで区切る
    for (key, sval) in memo_map.iter() {
        memory.push_str(&format!("{}: {} ", key, sval.num_format(2)));
    }
}

fn update_stack(stack: &VecDeque<CalcNum>, result: &mut String, decimal_point: usize) {
    // 改行で区切る。windowsの場合は\r\n, Mac|linuxの場合は\n
    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };
    for (i, sval) in stack.iter().enumerate().rev().take(10).rev() {
        result.push_str(sval.num_format(decimal_point).as_ref());
        if i < stack.len() - 1 {
            result.push_str(sepalator);
        }
    }
}
