// 1行入力欄 (カーソル編集と入力履歴)
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

const HISTORY_LIMIT: usize = 100;

#[derive(Default)]
pub struct InputLine {
    text: String,
    // カーソル位置 (文字数単位)
    cursor: usize,
    history: Vec<String>,
    // 履歴を辿っている間の位置。None は履歴を辿っていない状態
    history_pos: Option<usize>,
    // 履歴を辿り始める前に入力していた内容
    draft: String,
}

impl InputLine {
    pub fn text(&self) -> &str {
        &self.text
    }

    // カーソルより前の文字列 (カーソルの表示位置計算用)
    pub fn before_cursor(&self) -> &str {
        &self.text[..self.byte_index()]
    }

    pub fn cursor_at_end(&self) -> bool {
        self.cursor == self.char_len()
    }

    // カーソル直前の語 (直前の空白からカーソルまで)
    pub fn current_word(&self) -> &str {
        &self.text[self.word_start()..self.byte_index()]
    }

    // カーソル直前の語を置き換え、カーソルを置き換えた語の後ろへ移動する
    pub fn replace_current_word(&mut self, new: &str) {
        let (start, end) = (self.word_start(), self.byte_index());
        self.text.replace_range(start..end, new);
        self.cursor = self.text[..start].chars().count() + new.chars().count();
        self.history_pos = None;
    }

    fn word_start(&self) -> usize {
        let before = &self.text[..self.byte_index()];
        match before.char_indices().rfind(|(_, c)| c.is_whitespace()) {
            Some((i, c)) => i + c.len_utf8(),
            None => 0,
        }
    }

    // 入力内容を取り出して空にする
    pub fn take(&mut self) -> String {
        self.cursor = 0;
        self.history_pos = None;
        std::mem::take(&mut self.text)
    }

    pub fn clear(&mut self) {
        self.take();
    }

    pub fn push_history(&mut self, line: &str) {
        if self.history.last().map(String::as_str) != Some(line) {
            self.history.push(line.to_string());
            if self.history.len() > HISTORY_LIMIT {
                self.history.remove(0);
            }
        }
    }

    // 編集系のキー操作を処理する
    pub fn handle_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('a') if ctrl => self.cursor = 0,
            KeyCode::Char('e') if ctrl => self.cursor = self.char_len(),
            KeyCode::Char('b') if ctrl => self.move_left(),
            KeyCode::Char('f') if ctrl => self.move_right(),
            KeyCode::Char('u') if ctrl => self.kill_to_start(),
            KeyCode::Char('w') if ctrl => self.delete_word_before(),
            KeyCode::Char(c) if !ctrl => self.insert(c),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.char_len(),
            KeyCode::Up => self.history_prev(),
            KeyCode::Down => self.history_next(),
            _ => {}
        }
    }

    fn char_len(&self) -> usize {
        self.text.chars().count()
    }

    fn byte_index(&self) -> usize {
        self.text
            .char_indices()
            .nth(self.cursor)
            .map_or(self.text.len(), |(i, _)| i)
    }

    fn insert(&mut self, c: char) {
        let i = self.byte_index();
        self.text.insert(i, c);
        self.cursor += 1;
        self.history_pos = None;
    }

    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let i = self.byte_index();
            self.text.remove(i);
            self.history_pos = None;
        }
    }

    fn delete(&mut self) {
        if self.cursor < self.char_len() {
            let i = self.byte_index();
            self.text.remove(i);
            self.history_pos = None;
        }
    }

    fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.char_len());
    }

    fn kill_to_start(&mut self) {
        let i = self.byte_index();
        self.text.drain(..i);
        self.cursor = 0;
        self.history_pos = None;
    }

    // カーソル直前の単語を削除 (直前の空白も含めて)
    fn delete_word_before(&mut self) {
        let end = self.byte_index();
        let before = &self.text[..end];
        let start = before
            .trim_end()
            .rfind(char::is_whitespace)
            .map_or(0, |i| i + 1);
        let removed = self.text[start..end].chars().count();
        self.text.drain(start..end);
        self.cursor -= removed;
        self.history_pos = None;
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor = self.char_len();
    }

    fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let pos = match self.history_pos {
            None => {
                self.draft = self.text.clone();
                self.history.len() - 1
            }
            Some(p) => p.saturating_sub(1),
        };
        self.history_pos = Some(pos);
        self.set_text(self.history[pos].clone());
    }

    fn history_next(&mut self) {
        match self.history_pos {
            None => {}
            Some(p) if p + 1 < self.history.len() => {
                self.history_pos = Some(p + 1);
                self.set_text(self.history[p + 1].clone());
            }
            Some(_) => {
                // 最新より先に進んだら、辿る前の入力に戻す
                self.history_pos = None;
                let draft = std::mem::take(&mut self.draft);
                self.set_text(draft);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn type_str(input: &mut InputLine, s: &str) {
        for c in s.chars() {
            input.handle_key(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn edit_with_cursor() {
        let mut input = InputLine::default();
        type_str(&mut input, "13+");
        input.handle_key(key(KeyCode::Left));
        input.handle_key(key(KeyCode::Left));
        type_str(&mut input, " 2");
        assert_eq!(input.text(), "1 23+");
        assert_eq!(input.before_cursor(), "1 2");
        input.handle_key(key(KeyCode::Backspace));
        input.handle_key(key(KeyCode::Delete));
        assert_eq!(input.text(), "1 +");
        input.handle_key(ctrl('e'));
        assert_eq!(input.before_cursor(), "1 +");
    }

    #[test]
    fn multibyte_characters() {
        let mut input = InputLine::default();
        type_str(&mut input, "π2");
        input.handle_key(key(KeyCode::Left));
        input.handle_key(key(KeyCode::Backspace));
        assert_eq!(input.text(), "2");
    }

    #[test]
    fn delete_word_and_kill() {
        let mut input = InputLine::default();
        type_str(&mut input, "1 2 sqrt  ");
        input.handle_key(ctrl('w'));
        assert_eq!(input.text(), "1 2 ");
        input.handle_key(ctrl('u'));
        assert_eq!(input.text(), "");
    }

    #[test]
    fn current_word_and_replace() {
        let mut input = InputLine::default();
        type_str(&mut input, "2 sq");
        assert_eq!(input.current_word(), "sq");
        input.replace_current_word("sqrt ");
        assert_eq!(input.text(), "2 sqrt ");
        assert!(input.cursor_at_end());
        assert_eq!(input.current_word(), "");

        // カーソルが途中にある場合はカーソル直前までが対象
        let mut input = InputLine::default();
        type_str(&mut input, "ab cd");
        input.handle_key(key(KeyCode::Left));
        assert_eq!(input.current_word(), "c");
        input.replace_current_word("xyz");
        assert_eq!(input.text(), "ab xyzd");
        assert_eq!(input.before_cursor(), "ab xyz");
    }

    #[test]
    fn history_navigation() {
        let mut input = InputLine::default();
        input.push_history("1");
        input.push_history("2");
        input.push_history("2"); // 連続した重複は積まない
        type_str(&mut input, "dra");

        input.handle_key(key(KeyCode::Up));
        assert_eq!(input.text(), "2");
        input.handle_key(key(KeyCode::Up));
        assert_eq!(input.text(), "1");
        input.handle_key(key(KeyCode::Up)); // 先頭で止まる
        assert_eq!(input.text(), "1");
        input.handle_key(key(KeyCode::Down));
        assert_eq!(input.text(), "2");
        input.handle_key(key(KeyCode::Down)); // 入力途中の内容に戻る
        assert_eq!(input.text(), "dra");
    }
}
