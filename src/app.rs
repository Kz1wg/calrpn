// アプリケーションの状態とキー入力・コマンドの処理
// 描画は ui.rs が担当し、ここは端末に依存しないのでテストできる
use crate::calcrpn::{CalcState, DisplayFormat, Memorize, Notation, manage_stack};
use crate::commands;
use crate::input::InputLine;
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::cell::Cell;
use std::collections::VecDeque;
use std::io;

const UNDO_LIMIT: usize = 20;
const INPUT_LOG_LIMIT: usize = 8;
const MAX_DECIMAL_POINT: usize = 15;

pub enum Message {
    None,
    Info(String),
    Error(String),
    // Tab 補完の候補一覧など
    Hint(String),
}

pub enum Mode {
    Normal,
    Help { scroll: u16 },
}

pub struct App {
    pub state: CalcState,
    // sto / rcl などのキー待ち状態
    pub memo_mode: Option<Memorize>,
    pub format: DisplayFormat,
    pub input: InputLine,
    // 画面に表示する直近の入力
    pub input_log: VecDeque<String>,
    pub message: Message,
    pub mode: Mode,
    // ヘルプのスクロール上限 (表示領域の高さに依存するため描画時に ui が更新する)
    pub help_max_scroll: Cell<u16>,
    undo_history: VecDeque<CalcState>,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: CalcState::default(),
            memo_mode: None,
            format: DisplayFormat::default(),
            input: InputLine::default(),
            input_log: VecDeque::new(),
            message: Message::None,
            mode: Mode::Normal,
            help_max_scroll: Cell::new(0),
            undo_history: VecDeque::new(),
            should_quit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| ui::render(frame, self))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Windows では離したときのイベントも届くため、押下のみ扱う
        if key.kind != KeyEventKind::Press {
            return;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        if let Mode::Help { scroll } = &mut self.mode {
            let max = self.help_max_scroll.get();
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => *scroll = scroll.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => *scroll = (*scroll + 1).min(max),
                KeyCode::PageUp => *scroll = scroll.saturating_sub(10),
                KeyCode::PageDown => *scroll = scroll.saturating_add(10).min(max),
                KeyCode::Home => *scroll = 0,
                KeyCode::Esc | KeyCode::Enter | KeyCode::F(1) | KeyCode::Char('q') => {
                    self.mode = Mode::Normal;
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('d') if ctrl && self.input.text().is_empty() => self.should_quit = true,
            KeyCode::Char('z') if ctrl => self.undo(),
            KeyCode::F(1) => self.mode = Mode::Help { scroll: 0 },
            KeyCode::Esc => self.input.clear(),
            KeyCode::Tab => self.complete(),
            // 行末で → を押すと、薄く表示している補完候補を確定する
            KeyCode::Right if self.input.cursor_at_end() && self.ghost().is_some() => {
                self.complete();
            }
            KeyCode::Enter => {
                let line = self.input.take();
                self.submit(&line);
            }
            _ => self.input.handle_key(key),
        }
    }

    // 1行分の入力を処理する
    pub fn submit(&mut self, line: &str) {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            self.input.push_history(trimmed);
        }

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        match words[..] {
            ["undo"] => self.undo(),
            ["help"] => self.mode = Mode::Help { scroll: 0 },
            ["quit" | "q"] => self.should_quit = true,
            ["clh" | "clearhist"] => {
                self.input_log.clear();
                self.message = Message::Info("History cleared".to_string());
            }
            ["fix", arg] => self.set_notation(Notation::Fix, arg),
            ["sci", arg] => self.set_notation(Notation::Sci, arg),
            ["eng", arg] => self.set_notation(Notation::Eng, arg),
            ["sep"] => {
                self.format.grouping = !self.format.grouping;
                let state = if self.format.grouping { "on" } else { "off" };
                self.message = Message::Info(format!("Thousands separator: {state}"));
            }
            ["clv", key] => self.clear_memo(key),
            _ => self.calculate(line),
        }
    }

    // fix / sci / eng: 表記法と小数桁数を設定する
    fn set_notation(&mut self, notation: Notation, arg: &str) {
        match arg.parse::<usize>() {
            Ok(n) if n <= MAX_DECIMAL_POINT => {
                self.format.notation = notation;
                self.format.decimals = n;
                self.message = Message::Info(format!("{} {n}", notation.label()));
            }
            _ => {
                self.message = Message::Error(format!(
                    "{} requires 0..={MAX_DECIMAL_POINT}, got '{arg}'",
                    notation.label().to_lowercase()
                ));
            }
        }
    }

    fn clear_memo(&mut self, key: &str) {
        if self.state.memo_map.contains_key(key) {
            self.push_undo(self.state.clone());
            self.state.memo_map.remove(key);
            self.message = Message::Info(format!("Deleted memo '{key}'"));
        } else {
            self.message = Message::Error(format!("No memo key: '{key}'"));
        }
    }

    // カーソル直前の語に前方一致する補完候補 (ソート済み・重複なし)
    pub fn completions(&self) -> Vec<String> {
        let word = self.input.current_word();
        // 数値の入力中は補完しない
        if !word.chars().next().is_some_and(char::is_alphabetic) {
            return Vec::new();
        }
        let lower = word.to_lowercase();
        // コマンドは小文字で照合、メモのキーは大文字小文字を区別して照合
        let commands = commands::all_words()
            .filter(|w| w.starts_with(&lower) && *w != lower)
            .map(str::to_string);
        let keys = self
            .state
            .memo_map
            .keys()
            .filter(|k| k.starts_with(word) && k.as_str() != word)
            .cloned();
        let mut candidates: Vec<String> = commands.chain(keys).collect();
        candidates.sort();
        candidates.dedup();
        candidates
    }

    // 入力中に薄く表示する補完の残り部分 (候補が1つに絞れた場合のみ)
    pub fn ghost(&self) -> Option<String> {
        if !self.input.cursor_at_end() {
            return None;
        }
        match self.completions().as_slice() {
            [only] => only.get(self.input.current_word().len()..).map(str::to_string),
            _ => None,
        }
    }

    // Tab 補完: 候補が1つなら確定、複数なら共通部分まで補完して候補を表示
    fn complete(&mut self) {
        let candidates = self.completions();
        match candidates.as_slice() {
            [] => {}
            [only] => self.input.replace_current_word(&format!("{only} ")),
            _ => {
                let common = longest_common_prefix(&candidates);
                if common.len() > self.input.current_word().len() {
                    self.input.replace_current_word(&common);
                }
                self.message = Message::Hint(candidates.join("  "));
            }
        }
    }

    fn calculate(&mut self, line: &str) {
        let before = self.state.clone();
        let result = manage_stack(line, &mut self.state, &mut self.memo_mode);
        match result {
            Ok(()) => {
                self.log_input(line.trim());
                // 状態が変化した場合のみ、操作前の状態を Undo 履歴に積む
                if before != self.state {
                    self.push_undo(before);
                }
                self.message = Message::None;
            }
            Err(e) => self.message = Message::Error(e.to_string()),
        }
    }

    fn undo(&mut self) {
        match self.undo_history.pop_back() {
            Some(prev) => {
                self.state = prev;
                self.memo_mode = None;
                self.message =
                    Message::Info(format!("Undo (remaining: {})", self.undo_history.len()));
            }
            None => self.message = Message::Error("Nothing to undo".to_string()),
        }
    }

    pub fn undo_count(&self) -> usize {
        self.undo_history.len()
    }

    fn push_undo(&mut self, state: CalcState) {
        self.undo_history.push_back(state);
        if self.undo_history.len() > UNDO_LIMIT {
            self.undo_history.pop_front();
        }
    }

    fn log_input(&mut self, line: &str) {
        if line.is_empty() {
            return;
        }
        self.input_log.push_back(line.to_string());
        if self.input_log.len() > INPUT_LOG_LIMIT {
            self.input_log.pop_front();
        }
    }
}

fn longest_common_prefix(words: &[String]) -> String {
    let Some(first) = words.first() else {
        return String::new();
    };
    let mut prefix_len = first.len();
    for word in &words[1..] {
        prefix_len = first
            .char_indices()
            .zip(word.chars())
            .take_while(|((_, a), b)| a == b)
            .last()
            .map_or(0, |((i, c), _)| i + c.len_utf8())
            .min(prefix_len);
    }
    first[..prefix_len].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calcrpn::CalcNum;

    fn numbers(app: &App) -> Vec<f64> {
        app.state
            .stack
            .iter()
            .map(|v| match v {
                CalcNum::Number(n) => *n,
                CalcNum::Complex(_) => panic!("unexpected complex"),
            })
            .collect()
    }

    fn submit_all(app: &mut App, lines: &[&str]) {
        for line in lines {
            app.submit(line);
        }
    }

    #[test]
    fn undo_back_to_empty() {
        let mut app = App::new();
        submit_all(&mut app, &["1", "2", "+"]);
        assert_eq!(numbers(&app), [3.0]);
        app.submit("undo");
        assert_eq!(numbers(&app), [1.0, 2.0]);
        submit_all(&mut app, &["undo", "undo"]);
        assert!(app.state.stack.is_empty());
        app.submit("undo");
        assert!(matches!(app.message, Message::Error(_)));
    }

    #[test]
    fn undo_restores_memory() {
        let mut app = App::new();
        submit_all(&mut app, &["5 sto x"]);
        assert!(app.state.memo_map.contains_key("x"));
        app.submit("undo");
        assert!(app.state.memo_map.is_empty());

        submit_all(&mut app, &["5 sto x", "clv x"]);
        assert!(app.state.memo_map.is_empty());
        app.submit("undo");
        assert!(app.state.memo_map.contains_key("x"));
    }

    #[test]
    fn mode_only_input_is_not_pushed_to_undo() {
        let mut app = App::new();
        submit_all(&mut app, &["1", "sto"]);
        // 'sto' だけでは状態が変わらないので、undo で 1 の入力前に戻る
        app.submit("undo");
        assert!(app.state.stack.is_empty());
    }

    #[test]
    fn fix_command() {
        let mut app = App::new();
        app.submit("fix 5");
        assert_eq!(app.format.decimals, 5);
        assert!(matches!(app.message, Message::Info(_)));
        app.submit("fix abc");
        assert_eq!(app.format.decimals, 5);
        assert!(matches!(app.message, Message::Error(_)));
        app.submit("fix 99");
        assert_eq!(app.format.decimals, 5);
    }

    #[test]
    fn notation_and_separator_commands() {
        let mut app = App::new();
        app.submit("sci 2");
        assert_eq!(app.format.notation, Notation::Sci);
        assert_eq!(app.format.decimals, 2);
        app.submit("eng 4");
        assert_eq!(app.format.notation, Notation::Eng);
        app.submit("fix 3");
        assert_eq!(app.format.notation, Notation::Fix);

        assert!(app.format.grouping);
        app.submit("sep");
        assert!(!app.format.grouping);
        // 表示設定の変更はスタックに影響しない
        assert!(app.state.stack.is_empty());
    }

    #[test]
    fn error_keeps_stack_and_shows_message() {
        let mut app = App::new();
        submit_all(&mut app, &["1 2", "+ foo"]);
        assert_eq!(numbers(&app), [1.0, 2.0]);
        assert!(matches!(&app.message, Message::Error(m) if m.contains("foo")));
        // エラーになった入力はログに残らない
        assert_eq!(app.input_log, ["1 2"]);
    }

    #[test]
    fn two_numbers_are_not_app_command() {
        // 2語の入力でも fix / clv 以外は計算として扱う
        let mut app = App::new();
        app.submit("3 4");
        assert_eq!(numbers(&app), [3.0, 4.0]);
    }

    fn type_keys(app: &mut App, s: &str) {
        for c in s.chars() {
            app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
    }

    fn press(app: &mut App, code: KeyCode) {
        app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    #[test]
    fn tab_completion() {
        let mut app = App::new();

        // 候補が1つなら確定して空白を足す
        type_keys(&mut app, "2 sqr");
        assert_eq!(app.ghost().as_deref(), Some("t"));
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.input.text(), "2 sqrt ");

        // 候補が複数なら共通部分まで補完し、候補を表示
        app.input.clear();
        type_keys(&mut app, "rol");
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.input.text(), "roll");
        assert!(matches!(&app.message, Message::Hint(h) if h.contains("rollup")));
        assert_eq!(app.ghost(), None);

        // 行末の → でも確定できる
        app.input.clear();
        type_keys(&mut app, "las");
        press(&mut app, KeyCode::Right);
        assert_eq!(app.input.text(), "lastx ");

        // メモのキーも候補になる
        app.input.clear();
        app.submit("5 sto total");
        type_keys(&mut app, "tot");
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.input.text(), "total ");

        // 数値の入力中は補完しない
        app.input.clear();
        type_keys(&mut app, "10");
        assert!(app.completions().is_empty());
    }

    #[test]
    fn longest_common_prefix_works() {
        let words = ["rollup", "rolldown"].map(String::from);
        assert_eq!(longest_common_prefix(&words), "roll");
        let words = ["abc", "xyz"].map(String::from);
        assert_eq!(longest_common_prefix(&words), "");
        let words = ["same"].map(String::from);
        assert_eq!(longest_common_prefix(&words), "same");
    }

    #[test]
    fn undo_restores_last_x() {
        let mut app = App::new();
        submit_all(&mut app, &["9", "sqrt"]);
        assert!(app.state.last_x.is_some());
        app.submit("undo");
        assert!(app.state.last_x.is_none());
    }

    #[test]
    fn keys_drive_input_and_help() {
        let mut app = App::new();
        for c in "2 3*".chars() {
            app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(numbers(&app), [6.0]);
        assert_eq!(app.input.text(), "");

        app.handle_key(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE));
        assert!(matches!(app.mode, Mode::Help { .. }));
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(matches!(app.mode, Mode::Normal));

        app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert!(app.state.stack.is_empty());

        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit);
    }
}
