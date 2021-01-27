use simple_logger::SimpleLogger;
use std::io::{stdout, Write};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent, IME},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct TextareaState {
    text: Vec<char>,
    cursor_idx: usize,
    preedit: Option<PreeditState>,
}

struct PreeditState {
    text: String,
    start: usize,
    end: usize,
}

impl TextareaState {
    fn new() -> TextareaState {
        TextareaState {
            text: Vec::new(),
            cursor_idx: 0,
            preedit: None,
        }
    }
    fn validate_cusor_pos(&mut self) {
        self.cursor_idx = self.cursor_idx.max(0).min(self.text.len());
    }
    fn insert_before_cursor(&mut self, chr: char) {
        self.validate_cusor_pos();
        self.text.insert(self.cursor_idx, chr);
        self.cursor_idx += 1;
    }
    fn delete_before_cursor_if_exists(&mut self) {
        if (1..=self.text.len()).contains(&self.cursor_idx) {
            self.text.remove(self.cursor_idx - 1);
            self.cursor_idx -= 1;
        }
    }
    fn move_cursor_left(&mut self) {
        self.cursor_idx = self.cursor_idx.max(1) - 1;
    }
    fn move_cursor_right(&mut self) {
        self.cursor_idx = (self.cursor_idx + 1).min(self.text.len());
    }
    fn clear(&mut self) {
        self.text.clear();
        self.cursor_idx = 0;
        self.preedit = None;
    }
    fn draw_to_stdout(&self) {
        if self.text.is_empty() && self.preedit.is_none() {
            print!("\x1b[2mFocus the window and type something\x1b[0m");
        } else {
            let mut output = String::new();
            for idx in 0..=self.text.len() {
                if idx == self.cursor_idx {
                    if let Some(preedit) = &self.preedit {
                        let mut preedit_text = preedit.text.clone();
                        if preedit.start == preedit.end {
                            preedit_text.insert(preedit.end, '\u{2502}');
                        } else {
                            preedit_text.insert_str(preedit.end, "\x1b[0m\x1b[4m");
                            preedit_text.insert_str(preedit.start, "\x1b[7m");
                        }
                        output.push_str("\x1b[4m");
                        output.push_str(&preedit_text);
                        output.push_str("\x1b[0m");
                    } else {
                        output.push('\u{2502}');
                    }
                }
                if idx < self.text.len() {
                    output.push(self.text[idx].clone());
                }
            }
            print!("{}", output);
        }
        stdout().flush().unwrap();
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
    textarea.draw_to_stdout();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::ReceivedCharacter(codepoint) => {
                    textarea.preedit = None; // On linux, Commit event comes after ReceivedCharacter
                    match codepoint {
                        '\u{7F}' => textarea.clear(),
                        '\u{08}' => textarea.delete_before_cursor_if_exists(),
                        '\u{0}'..='\u{1F}' => (), //Other control sequence
                        chr => textarea.insert_before_cursor(chr),
                    }
                    print!("\x1b[F\x1b[E\x1b[K");
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
                        VirtualKeyCode::Left => textarea.move_cursor_left(),
                        VirtualKeyCode::Right => textarea.move_cursor_right(),
                        _ => (),
                    }
                    print!("\x1b[F\x1b[E\x1b[K");
                    println!("{:?}", event);
                    textarea.draw_to_stdout();
                }
                WindowEvent::IME(event) => {
                    textarea.preedit = None;
                    print!("\x1b[F\x1bE\x1b[K");
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
