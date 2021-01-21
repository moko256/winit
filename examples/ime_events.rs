use std::io::{stdout, Write};
use simple_logger::SimpleLogger;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct TextareaState {
    text: Vec<char>,
    cursor_idx: usize,
    preedit_start: Option<usize>,
    preedit_end: Option<usize>,
}

impl TextareaState {
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
            self.text.remove(self.cursor_idx-1);
            self.cursor_idx -= 1;
        }
    }
    fn move_cursor_left(&mut self) {
        self.cursor_idx = self.cursor_idx.max(1)-1;
    }
    fn move_cursor_right(&mut self) {
        self.cursor_idx = (self.cursor_idx+1).min(self.text.len());
    }
    fn clear(&mut self) {
        self.text.clear();
        self.cursor_idx = 0;
        self.preedit_start = None;
        self.preedit_end = None;
    }
    fn draw_to_stdout(&self) {
        let mut output = String::new();
        for (idx, chr) in self.text.iter().enumerate() {
            if idx == self.cursor_idx {
                output.push('\u{2502}');
            }
            output.push(chr.clone());
        }
        if self.text.len() == self.cursor_idx {
            output.push('\u{2502}');
        }
        print!("{}", output);
        stdout().flush().unwrap();
    }
}

fn main() {
    println!("\u{1f469}\u{1f3fb}こんにちは\x1b[48;2;255;255;255m\x1b[30m今日\x1b[0m\u{2502}");
    SimpleLogger::new().init().unwrap();
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(winit::dpi::LogicalSize::new(128.0, 128.0))
        .build(&event_loop)
        .unwrap();
    window.set_ime_position(PhysicalPosition::new(0.0, 0.0));
    
    print!("Focus the window and type something");
    stdout().flush().unwrap();
    
    let mut textarea = TextareaState {
        text: Vec::new(),
        cursor_idx: 0,
        preedit_start: None,
        preedit_end: None,
    };
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(codepoint),
                ..
            } => {
                print!("\n\x1b[F\x1b[K");
                println!("{} : {}", codepoint, codepoint.escape_unicode());
                match codepoint {
                    '\u{7F}' => textarea.clear(),
                    '\u{08}' => textarea.delete_before_cursor_if_exists(),
                    chr => textarea.insert_before_cursor(chr),
                }
                textarea.draw_to_stdout();
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(state),
                        ..
                    },
                    ..
                },
                ..
            } => {
                if state == VirtualKeyCode::Left || state == VirtualKeyCode::Right {
                    print!("\x1b[F\x1b[E\x1b[K");
                    match state {
                        VirtualKeyCode::Left => {
                            textarea.move_cursor_left();
                        }
                        VirtualKeyCode::Right => {
                            textarea.move_cursor_right();
                        }
                        _ => (),
                    }
                    textarea.draw_to_stdout();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
