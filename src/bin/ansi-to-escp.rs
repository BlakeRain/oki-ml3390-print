use std::io::BufRead;

use ansi_str::{get_blocks, Style};

const CODE_BOLD_ENABLE: &str = "\x1bE";
const CODE_BOLD_DISABLE: &str = "\x1bF";
const CODE_ITALIC_ENABLE: &str = "\x1b4";
const CODE_ITALIC_DISABLE: &str = "\x1b5";
const CODE_UNDERLINE_ENABLE: &str = "\x1b-1";
const CODE_UNDERLINE_DISABLE: &str = "\x1b-0";

#[derive(Default)]
struct State {
    bold: bool,
    italic: bool,
    underline: bool,
}

macro_rules! style_accessors {
    ($set_name:ident, $clear_name:ident, $field:ident, $enable:expr, $disable:expr) => {
        fn $set_name(&mut self) {
            if !self.$field {
                print!("{}", $enable);
            }

            self.$field = true;
        }

        fn $clear_name(&mut self) {
            if self.$field {
                print!("{}", $disable);
            }

            self.$field = false;
        }
    };
}

impl State {
    style_accessors!(
        set_bold,
        clear_bold,
        bold,
        CODE_BOLD_ENABLE,
        CODE_BOLD_DISABLE
    );

    style_accessors!(
        set_italic,
        clear_italic,
        italic,
        CODE_ITALIC_ENABLE,
        CODE_ITALIC_DISABLE
    );

    style_accessors!(
        set_underline,
        clear_underline,
        underline,
        CODE_UNDERLINE_ENABLE,
        CODE_UNDERLINE_DISABLE
    );

    fn set(mut self, style: &Style) -> Self {
        if style.is_bold() {
            self.set_bold();
        } else {
            self.clear_bold();
        }

        if style.is_italic() {
            self.set_italic();
        } else {
            self.clear_italic();
        }

        if style.is_underline() {
            self.set_underline();
        } else {
            self.clear_underline();
        }

        self
    }

    fn unset(self) {
        if self.bold {
            print!("{CODE_BOLD_DISABLE}")
        }

        if self.italic {
            print!("{CODE_ITALIC_DISABLE}")
        }

        if self.underline {
            print!("{CODE_UNDERLINE_DISABLE}")
        }
    }
}

fn main() {
    let content = {
        let mut content = String::new();
        for buffer in std::io::stdin().lock().lines() {
            match buffer {
                Err(err) => panic!("Unable to read from stdin: {:?}", err),
                Ok(buffer) => {
                    content.push_str(&buffer);
                }
            }
        }

        content
    };

    for block in get_blocks(&content) {
        let state = State::default().set(block.style());
        print!("{}", block.text());
        state.unset();
    }
}
