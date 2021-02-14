use simple_logger::SimpleLogger;
use std::io::{stdout, Write};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent, IME},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct TextareaState {
    text: String,
    cursor_idx: usize,
    preedit: Option<PreeditState>,
    hint: String,
    focusing: bool,
}

struct PreeditState {
    text: String,
    start: usize,
    end: usize,
}

impl TextareaState {
    fn new() -> TextareaState {
        TextareaState {
            text: String::new(),
            cursor_idx: 0,
            preedit: None,
            hint: String::new(),
            focusing: true,
        }
    }
    fn insert_to_cursor_left(&mut self, ch: char) {
        self.text.insert(self.cursor_idx, ch);
        self.cursor_idx += ch.len_utf8();
    }
    fn move_cursor_to_left(&mut self) {
        let left_ch = &self.text[..self.cursor_idx].chars().next_back();
        if let Some(ch) = left_ch {
            self.cursor_idx -= ch.len_utf8();
        }
    }
    fn move_cursor_to_right(&mut self) {
        let right_ch = &self.text[self.cursor_idx..].chars().next();
        if let Some(ch) = right_ch {
            self.cursor_idx += ch.len_utf8();
        }
    }
    fn delete_cursor_left(&mut self) {
        let left_ch = &self.text[..self.cursor_idx].chars().next_back();
        if let Some(ch) = left_ch {
            self.cursor_idx -= ch.len_utf8();
            self.text.remove(self.cursor_idx);
        }
    }
    fn delete_cursor_right(&mut self) {
        let right_ch = &self.text[self.cursor_idx..].chars().next();
        if let Some(_) = right_ch {
            self.text.remove(self.cursor_idx);
        }
    }
    fn clear(&mut self) {
        self.text.clear();
        self.cursor_idx = 0;
        self.preedit = None;
    }
    fn draw_to_stdout(&self) {
        if self.text.is_empty() && self.preedit.is_none() {
            if self.focusing {
                print!("\x1b[2m{}\x1b[0m", self.hint);
            } else {
                print!("\x1b[1mFocus the window\x1b[0m");
            }
        } else {
            let mut output = self.text.clone();
            if let Some(preedit) = &self.preedit {
                // insertion in this block is reverse order
                output.insert_str(self.cursor_idx, "\x1b[0m");
                output.insert_str(self.cursor_idx, &preedit.text);
                if preedit.start == preedit.end {
                    output.insert(self.cursor_idx + preedit.start, '\u{2502}');
                } else {
                    output.insert_str(self.cursor_idx + preedit.start, "\x1b[0m\x1b[4m");
                    output.insert_str(self.cursor_idx + preedit.start, "\x1b[7m");
                }
                output.insert_str(self.cursor_idx, "\x1b[4m");
            } else if self.focusing {
                output.insert(self.cursor_idx, '\u{2502}');
            }
            print!("{}", output);
        }
        stdout().flush().unwrap();
        print!("\x1b[F\x1b[E\x1b[K");
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
        .build(&event_loop)
        .unwrap();

    let mut textarea = TextareaState::new();
    textarea.hint = "Type something...".to_string();
    textarea.draw_to_stdout();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::ReceivedCharacter(codepoint) => {
                    textarea.preedit = None; // On linux, Commit event comes after ReceivedCharacter
                    match codepoint {
                        '\u{08}' => textarea.delete_cursor_left(),
                        '\u{7F}' => textarea.delete_cursor_right(),
                        '\r' | '\n' => textarea.clear(),
                        '\u{0}'..='\u{1F}' => (), //Other control sequence
                        ch => textarea.insert_to_cursor_left(ch),
                    }
                    println!("{:?}", event);
                    textarea.draw_to_stdout();
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(state),
                            ..
                        },
                    ..
                } if state == VirtualKeyCode::Left || state == VirtualKeyCode::Right => {
                    match state {
                        VirtualKeyCode::Left => textarea.move_cursor_to_left(),
                        VirtualKeyCode::Right => textarea.move_cursor_to_right(),
                        _ => (),
                    }
                    println!("{:?}", event);
                    textarea.draw_to_stdout();
                }
                WindowEvent::IME(event) => {
                    textarea.preedit = None;
                    println!("{:?}", event);
                    match event {
                        IME::Enabled => window.set_ime_position(PhysicalPosition::new(0.0, 0.0)),
                        IME::Preedit(t, s, e) => {
                            textarea.preedit = Some(PreeditState {
                                text: t.clone(),
                                start: s.unwrap_or(0),
                                end: e.unwrap_or(t.len()),
                            });
                        }
                        _ => (),
                    }
                    textarea.draw_to_stdout();
                }
                WindowEvent::Focused(focusing) => {
                    textarea.focusing = focusing;
                    println!("{:?}", event);
                    textarea.draw_to_stdout();
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    println!("");
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
