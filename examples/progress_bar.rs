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
mod utils;
use utils::Loader;

use std::time::Duration;

use tui_realm_stdlib::ProgressBar;
use tuirealm::command::CmdResult;
use tuirealm::listener::{ListenerResult, Poll};
use tuirealm::props::{
    Alignment, AttrValue, Attribute, BorderType, Borders, Color, PropPayload, PropValue,
};
use tuirealm::terminal::TerminalBridge;
use tuirealm::{
    application::PollStrategy,
    event::{Key, KeyEvent},
    Application, Component, Event, EventListenerCfg, MockComponent, Update, View,
};
// tui
use tuirealm::tui::layout::{Constraint, Direction as LayoutDirection, Layout};

#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    GaugeAlfaBlur,
    GaugeBetaBlur,
    None,
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    GaugeAlfa,
    GaugeBeta,
}

#[derive(PartialEq, Clone, PartialOrd)]
enum UserEvent {
    Loaded(f64),
}

impl Eq for UserEvent {}

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
    fn view(&mut self, app: &mut Application<Id, Msg, UserEvent>) {
        let _ = self.terminal.raw_mut().draw(|f| {
            // Prepare chunks
            let chunks = Layout::default()
                .direction(LayoutDirection::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            app.view(&Id::GaugeAlfa, f, chunks[0]);
            app.view(&Id::GaugeBeta, f, chunks[1]);
        });
    }
}

fn main() {
    let mut model = Model::default();
    let _ = model.terminal.enable_raw_mode();
    let _ = model.terminal.enter_alternate_screen();
    // Setup app
    let mut app: Application<Id, Msg, UserEvent> = Application::init(
        EventListenerCfg::default()
            .default_input_listener(Duration::from_millis(10))
            .port(Box::new(Loader::default()), Duration::from_millis(50)),
    );
    assert!(app
        .mount(Id::GaugeAlfa, Box::new(GaugeAlfa::default()), vec![])
        .is_ok());
    assert!(app
        .mount(Id::GaugeBeta, Box::new(GaugeBeta::default()), vec![])
        .is_ok());
    // We need to give focus to input then
    assert!(app.active(&Id::GaugeAlfa).is_ok());
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

impl Update<Id, Msg, UserEvent> for Model {
    fn update(&mut self, view: &mut View<Id, Msg, UserEvent>, msg: Option<Msg>) -> Option<Msg> {
        match msg.unwrap_or(Msg::None) {
            Msg::AppClose => {
                self.quit = true;
                None
            }
            Msg::GaugeAlfaBlur => {
                assert!(view.active(&Id::GaugeBeta).is_ok());
                None
            }
            Msg::GaugeBetaBlur => {
                assert!(view.active(&Id::GaugeAlfa).is_ok());
                None
            }
            Msg::None => None,
        }
    }
}

// -- poll

impl Poll<UserEvent> for Loader {
    fn poll(&mut self) -> ListenerResult<Option<Event<UserEvent>>> {
        Ok(Some(Event::User(UserEvent::Loaded(self.load()))))
    }
}

// -- components

#[derive(MockComponent)]
struct GaugeAlfa {
    component: ProgressBar,
}

impl Default for GaugeAlfa {
    fn default() -> Self {
        Self {
            component: ProgressBar::default()
                .borders(
                    Borders::default()
                        .color(Color::Green)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Green)
                .label("0%")
                .title("Loading...", Alignment::Center)
                .progress(0.0),
        }
    }
}

impl Component<Msg, UserEvent> for GaugeAlfa {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::User(UserEvent::Loaded(prog)) => {
                // Update
                let label = format!("{:02}%", (prog * 100.0) as usize);
                self.attr(
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::One(PropValue::F64(prog))),
                );
                self.attr(Attribute::Text, AttrValue::String(label));
                CmdResult::None
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::GaugeAlfaBlur),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
struct GaugeBeta {
    component: ProgressBar,
}

impl Default for GaugeBeta {
    fn default() -> Self {
        Self {
            component: ProgressBar::default()
                .borders(
                    Borders::default()
                        .color(Color::Yellow)
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Yellow)
                .label("0%")
                .title("Loading...", Alignment::Center)
                .progress(0.0),
        }
    }
}

impl Component<Msg, UserEvent> for GaugeBeta {
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::User(UserEvent::Loaded(prog)) => {
                // Update
                let label = format!("{:02}%", (prog * 100.0) as usize);
                self.attr(
                    Attribute::Value,
                    AttrValue::Payload(PropPayload::One(PropValue::F64(prog))),
                );
                self.attr(Attribute::Text, AttrValue::String(label));
                CmdResult::None
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::GaugeBetaBlur),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}
