//! ## Demo
//!
//! `Demo` shows how to use tui-realm in a real case

/**
 * MIT License
 *
 * tui-realm - Copyright (C) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use std::time::Duration;

use tui_realm_stdlib::Select;
use tuirealm::command::{Cmd, CmdResult, Direction};
use tuirealm::props::{Alignment, BorderType, Borders, Color};
use tuirealm::terminal::TerminalBridge;
use tuirealm::State;
use tuirealm::{
    application::PollStrategy,
    event::{Key, KeyEvent},
    Application, Component, Event, EventListenerCfg, MockComponent, NoUserEvent, Update, View,
};
// tui
use tuirealm::tui::layout::{Constraint, Direction as LayoutDirection, Layout};

#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    SelectAlfaBlur,
    SelectBetaBlur,
    None,
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    SelectAlfa,
    SelectBeta,
}

struct Model {
    quit: bool,   // Becomes true when the user presses <ESC>
    redraw: bool, // Tells whether to refresh the UI; performance optimization
    terminal: TerminalBridge,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            quit: false,
            redraw: true,
            terminal: TerminalBridge::new().expect("Cannot create terminal bridge"),
        }
    }
}

impl Model {
    fn view(&mut self, app: &mut Application<Id, Msg, NoUserEvent>) {
        // Calc len
        let select_alfa_len = match app.state(&Id::SelectAlfa) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let select_beta_len = match app.state(&Id::SelectBeta) {
            Ok(State::One(_)) => 3,
            _ => 8,
        };
        let _ = self.terminal.raw_mut().draw(|f| {
            // Prepare chunks
            let chunks = Layout::default()
                .direction(LayoutDirection::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(select_alfa_len),
                        Constraint::Length(select_beta_len),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            app.view(&Id::SelectAlfa, f, chunks[0]);
            app.view(&Id::SelectBeta, f, chunks[1]);
        });
    }
}

fn main() {
    let mut model = Model::default();
    let _ = model.terminal.enable_raw_mode();
    let _ = model.terminal.enter_alternate_screen();
    // Setup app
    let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
        EventListenerCfg::default().default_input_listener(Duration::from_millis(10)),
    );
    assert!(app
        .mount(Id::SelectAlfa, Box::new(SelectAlfa::default()), vec![])
        .is_ok());
    assert!(app
        .mount(Id::SelectBeta, Box::new(SelectBeta::default()), vec![])
        .is_ok());
    // We need to give focus to input then
    assert!(app.active(&Id::SelectAlfa).is_ok());
    // Now we use the Model struct to keep track of some states

    // let's loop until quit is true
    while !model.quit {
        // Tick
        if let Ok(sz) = app.tick(&mut model, PollStrategy::Once) {
            if sz > 0 {
                // NOTE: redraw if at least one msg has been processed
                model.redraw = true;
            }
        }
        // Redraw
        if model.redraw {
            model.view(&mut app);
            model.redraw = false;
        }
    }
    // Terminate terminal
    let _ = model.terminal.leave_alternate_screen();
    let _ = model.terminal.disable_raw_mode();
    let _ = model.terminal.clear_screen();
}

impl Update<Id, Msg, NoUserEvent> for Model {
    fn update(&mut self, view: &mut View<Id, Msg, NoUserEvent>, msg: Option<Msg>) -> Option<Msg> {
        match msg.unwrap_or(Msg::None) {
            Msg::AppClose => {
                self.quit = true;
                None
            }
            Msg::SelectAlfaBlur => {
                assert!(view.active(&Id::SelectBeta).is_ok());
                None
            }
            Msg::SelectBetaBlur => {
                assert!(view.active(&Id::SelectAlfa).is_ok());
                None
            }
            Msg::None => None,
        }
    }
}

#[derive(MockComponent)]
struct SelectAlfa {
    component: Select,
}

impl Default for SelectAlfa {
    fn default() -> Self {
        Self {
            component: Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::LightGreen),
                )
                .foreground(Color::LightGreen)
                .background(Color::Black)
                .title("Select your ice cream flavour 🍦", Alignment::Center)
                .rewind(true)
                .highlighted_color(Color::LightGreen)
                .highlighted_str(">> ")
                .choices(&[
                    "vanilla",
                    "chocolate",
                    "coconut",
                    "mint",
                    "strawberry",
                    "lemon",
                    "hazelnut",
                    "coffee",
                ]),
        }
    }
}

impl Component<Msg, NoUserEvent> for SelectAlfa {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::SelectAlfaBlur),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
struct SelectBeta {
    component: Select,
}

impl Default for SelectBeta {
    fn default() -> Self {
        Self {
            component: Select::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::LightYellow),
                )
                .foreground(Color::LightYellow)
                .background(Color::Black)
                .title("Select your topping 🧁", Alignment::Center)
                .rewind(false)
                .highlighted_color(Color::LightYellow)
                .highlighted_str(">> ")
                .choices(&[
                    "hazelnuts",
                    "chocolate",
                    "maple cyrup",
                    "smarties",
                    "raspberries",
                ]),
        }
    }
}

impl Component<Msg, NoUserEvent> for SelectBeta {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::SelectBetaBlur),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}
