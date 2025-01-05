// https://github.com/ingobeans/banano

use cool_rust_input::{
    set_terminal_line, CoolInput, CustomInputHandler, HandlerContext, InputTransform,
    KeyPressResult,
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::style::{ResetColor, SetBackgroundColor};
use crossterm::{
    queue,
    style::{Color, SetForegroundColor},
};
use dialoguer::{Confirm, Input};
use std::env;
use std::fs;
use std::io::{stdin, stdout, Write};

fn save_file(filename: &str, text: &str) {
    fs::write(filename, text).expect("Unable to write new contents.");
}
pub struct FileEditorInput {
    pub filename: String,
    original_text: String,
    is_new: bool,
}
impl FileEditorInput {
    fn open_filename(filename: String, original_text: String, is_new: bool) -> Self {
        FileEditorInput {
            filename,
            original_text,
            is_new,
        }
    }
}
impl CustomInputHandler for FileEditorInput {
    fn handle_key_press(&mut self, key: &Event, ctx: HandlerContext) -> KeyPressResult {
        if let Event::Key(key_event) = key {
            if key_event.kind == crossterm::event::KeyEventKind::Press {
                // Disallow the user pressing the character 'S'
                if let KeyCode::Char(c) = key_event.code {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        if c == 'c' {
                            return KeyPressResult::Stop;
                        }
                        if c == 's' {
                            save_file(&self.filename, &ctx.text_data.text);
                            self.is_new = false;
                            self.original_text = ctx.text_data.text.to_owned();
                            return KeyPressResult::Handled;
                        }
                    }
                }
            }
        }
        KeyPressResult::Continue
    }
    fn after_draw_text(&mut self, ctx: HandlerContext) {
        let _ = queue!(
            stdout(),
            SetForegroundColor(Color::Black),
            SetBackgroundColor(Color::White)
        );
        let left_text = format!("BANANO v{}", env!("CARGO_PKG_VERSION"));
        let center_text = format!("FILE: '{}'", self.filename);
        let mut right_text = "NOT MODIFIED";

        if self.original_text != ctx.text_data.text {
            right_text = "MODIFIED";
        }
        if self.is_new {
            right_text = "NEW FILE"
        }

        let bottom_text_position = (ctx.terminal_size.1 - 1) as usize;
        let width = self.get_input_transform(ctx).size.0;

        let _ = set_terminal_line(&left_text, 0, 0, true);
        let _ = set_terminal_line(
            &center_text,
            (width as usize - center_text.len()) / 2,
            0,
            false,
        );
        let _ = set_terminal_line(right_text, width as usize - right_text.len(), 0, false);

        let keybinds = ["^S".to_string(), "^C".to_string()];
        let descriptions = ["Save File".to_string(), "Exit".to_string()];

        let mut offset = 0;
        for (keybind, description) in keybinds.iter().zip(descriptions) {
            let _ = queue!(
                stdout(),
                SetForegroundColor(Color::Black),
                SetBackgroundColor(Color::White)
            );
            let _ = set_terminal_line(keybind, offset, bottom_text_position, false);
            offset += keybind.chars().count() + 1;
            let _ = queue!(stdout(), ResetColor);
            let _ = set_terminal_line(&description, offset, bottom_text_position, false);
            offset += description.chars().count() + 1;
        }
    }
    fn get_input_transform(&mut self, ctx: HandlerContext) -> InputTransform {
        let size = (ctx.terminal_size.0, ctx.terminal_size.1 - 3);
        let offset = (0, 2);
        InputTransform { size, offset }
    }
}

pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("please specify a filename!");
        return Ok(());
    }
    let filename = &args[1];
    let mut text = String::new();
    let mut is_new = true;
    if path_exists(filename) {
        text = fs::read_to_string(filename).expect("Unable to read file contents.");
        is_new = false;
    }
    let mut cool_input = CoolInput::new(
        FileEditorInput::open_filename(filename.to_string(), text.to_owned(), is_new),
        0,
    );
    cool_input.text_data.text = text;
    cool_input.listen()?;
    if cool_input.custom_input.original_text != cool_input.text_data.text {
        let save = Confirm::new().with_prompt("Save file?").interact().unwrap();
        if save {
            save_file(&filename, &cool_input.text_data.text);
        }
    }
    Ok(())
}
