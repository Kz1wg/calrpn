mod calcrpn;
mod commands;
mod finance;
use calcrpn::{CalcNum, DegMode, Memorize, manage_stack};
use crossterm::execute;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};
use rustyline::{DefaultEditor, config::Configurer};
use std::{
    collections::{BTreeMap, VecDeque},
    io,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
    let mut stack: VecDeque<CalcNum> = VecDeque::new();
    let mut memo_map: BTreeMap<String, CalcNum> = BTreeMap::new();
    let mut degmode = DegMode::Deg;
    let mut memo_mode: Option<Memorize> = None;
    let mut result = String::new();
    let mut memory = String::new();
    let mut message = String::new();
    let mut decimal_point: usize = 3;
    let mut undo_history: VecDeque<Snapshot> = VecDeque::new();
    let mut do_continue = true;
    let mut input = String::new();
    let mut readline = DefaultEditor::new()?;
    let mut input_log: VecDeque<String> = VecDeque::new();

    readline.set_max_history_size(20)?;

    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };

    while do_continue {
        loop {
            let result_len = calcrpn::STACK_SIZE.min(stack.len()) + 2;
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
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.area());
                // ステータスバー
                let status_block = Block::default().borders(Borders::NONE);
                let status_text = Paragraph::new(format!(
                    "Fix:{decimal_point} | {degmode:?} | MemoMode:{memo_mode:?}",
                ))
                .block(status_block);

                f.render_widget(status_text, chunks[0]);

                //Memory
                let memory_block = Block::default().borders(Borders::NONE);
                let memory_text = Paragraph::new(memory.clone()).block(memory_block);

                f.render_widget(memory_text, chunks[1]);
                // Helper メッセージ
                let help_block = Block::default().title("Message").borders(Borders::ALL);
                let help_text = Paragraph::new(format!(
                    "Enter: Calc | quit or q : Quit | Undo : undo{sepalator}{message}",
                ))
                .block(help_block);

                f.render_widget(help_text, chunks[2]);

                //結果表示
                let result_block = Block::default().title("Result Stack").borders(Borders::ALL);
                let result_text = Paragraph::new(result.clone()).block(result_block);
                f.render_widget(result_text, chunks[3]);

                // 入力
                let input_block = Block::default().title("Input").borders(Borders::TOP);
                let input_text = Paragraph::new(input.clone()).block(input_block);
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
            if !input.is_empty() {
                input_log.push_back(input.clone());
            }

            match input.trim() {
                "undo" => {
                    // undoの処理: 直前の操作の前の状態に戻す
                    input_log.pop_back();
                    if let Some(prev) = undo_history.pop_back() {
                        prev.restore(&mut stack, &mut memo_map, &mut degmode);
                        memo_mode = None;
                        update_stack(&stack, &mut result, decimal_point);
                        update_memo(&memo_map, &mut memory);
                        message = format!("Undo (remaining: {})", undo_history.len());
                    } else {
                        message = "Nothing to undo".to_string();
                    }
                    terminal.clear()?;
                    continue;
                }
                "help" => {
                    break;
                }
                "quit" | "q" => {
                    do_continue = false;
                    break;
                }
                "clh" | "clearhist" => {
                    input_log.clear();
                    terminal.clear()?;
                }
                _ => {
                    let app_command = input.split_whitespace().collect::<Vec<&str>>();
                    // 'fix 2'のように引数を取るアプリコマンド
                    // 処理した場合は計算(manage_stack)へ渡さずに次の入力へ
                    if let [command, arg] = app_command[..] {
                        match command {
                            "fix" => {
                                match arg.parse::<usize>() {
                                    Ok(fix) => {
                                        decimal_point = fix;
                                        readline.add_history_entry(&input)?;
                                        update_log(&mut input_log, &mut message);
                                    }
                                    Err(_) => {
                                        message = format!("Error: invalid fix value '{arg}'");
                                        input_log.pop_back();
                                    }
                                }
                                update_stack(&stack, &mut result, decimal_point);
                                terminal.clear()?;
                                continue;
                            }
                            "clv" => {
                                // 特定のmemo keyを削除
                                if memo_map.contains_key(arg) {
                                    push_undo(
                                        &mut undo_history,
                                        Snapshot::capture(&stack, &memo_map, degmode),
                                    );
                                    memo_map.remove(arg);
                                }
                                readline.add_history_entry(&input)?;
                                update_log(&mut input_log, &mut message);
                                update_memo(&memo_map, &mut memory);
                                terminal.clear()?;
                                continue;
                            }
                            _ => (),
                        }
                    }
                    let pre_stack_length = stack.len();
                    let before = Snapshot::capture(&stack, &memo_map, degmode);

                    match manage_stack(
                        &input,
                        &mut stack,
                        &mut degmode,
                        &mut memo_map,
                        &mut memo_mode,
                    ) {
                        Ok(()) => {
                            // 入力を履歴に追加
                            readline.add_history_entry(&input)?;
                            update_log(&mut input_log, &mut message);
                            // 状態が変化した場合のみ、操作前の状態をUndo履歴に積む
                            // ('sto' 単体入力などモード変更だけの操作は積まない)
                            if !before.is_same_as(&stack, &memo_map, degmode) {
                                push_undo(&mut undo_history, before);
                            }
                        }
                        Err(e) => {
                            message = format!("Error: {e}");
                            input_log.pop_back();
                        }
                    }
                    if pre_stack_length > stack.len() {
                        terminal.clear()?;
                    }
                    update_stack(&stack, &mut result, decimal_point);
                    update_memo(&memo_map, &mut memory);
                }
            }
        }

        if do_continue {
            ratatui::restore();
            calcrpn::print_help();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            match input.trim() {
                "quit" | "q" => {
                    terminal.clear()?;
                    ratatui::restore();
                    break;
                }
                _ => terminal.clear()?,
            }
            *terminal = ratatui::init();
        } else {
            break;
        }
    }
    Ok(())
}

const UNDO_LIMIT: usize = 20;

// Undo用の状態スナップショット(操作前の状態を保存する)
struct Snapshot {
    stack: VecDeque<CalcNum>,
    memo_map: BTreeMap<String, CalcNum>,
    degmode: DegMode,
}

impl Snapshot {
    fn capture(
        stack: &VecDeque<CalcNum>,
        memo_map: &BTreeMap<String, CalcNum>,
        degmode: DegMode,
    ) -> Self {
        Self {
            stack: stack.clone(),
            memo_map: memo_map.clone(),
            degmode,
        }
    }

    fn is_same_as(
        &self,
        stack: &VecDeque<CalcNum>,
        memo_map: &BTreeMap<String, CalcNum>,
        degmode: DegMode,
    ) -> bool {
        self.stack == *stack && self.memo_map == *memo_map && self.degmode == degmode
    }

    fn restore(
        self,
        stack: &mut VecDeque<CalcNum>,
        memo_map: &mut BTreeMap<String, CalcNum>,
        degmode: &mut DegMode,
    ) {
        *stack = self.stack;
        *memo_map = self.memo_map;
        *degmode = self.degmode;
    }
}

fn push_undo(undo_history: &mut VecDeque<Snapshot>, snapshot: Snapshot) {
    undo_history.push_back(snapshot);
    if undo_history.len() > UNDO_LIMIT {
        undo_history.pop_front();
    }
}

fn update_log(input_log: &mut VecDeque<String>, message: &mut String) {
    if input_log.len() > 8 {
        input_log.pop_front();
    }
    *message = input_log
        .iter()
        .rev()
        .fold("Hist: ".to_string(), |acc, x| acc + x + " → ");
}

fn update_memo(memo_map: &BTreeMap<String, CalcNum>, memory: &mut String) {
    // memo_mapをStringに変換しスペースで区切る(毎回作り直す)
    *memory = memo_map
        .iter()
        .map(|(key, sval)| format!("{} -> {}", key, sval.num_format(2)))
        .collect::<Vec<_>>()
        .join(" ");
}

fn update_stack(stack: &VecDeque<CalcNum>, result: &mut String, decimal_point: usize) {
    // 改行で区切る。windowsの場合は\r\n, Mac|linuxの場合は\n
    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };
    *result = stack
        .iter()
        .rev()
        .take(calcrpn::STACK_SIZE)
        .rev()
        .map(|x| x.num_format(decimal_point))
        .collect::<Vec<_>>()
        .join(sepalator);
}
