/*
Copyright 2021 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

#![windows_subsystem = "windows"]

use cubit::core::{Application, Game, Window};
use cubit::graphics::Color;
use cubit::os::Os;
use cubit::ApplicationComponent;
use cubit::StartupListener;
use lockjaw::{component_visible, injectable, module, Cl};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::time::{Duration, Instant};

lockjaw::prologue!("src/main.rs");

struct GameState<'a> {
    frame_counter: i32,
    time_start: Instant,
    window: Option<Rc<dyn Window + 'a>>,
    t: f32,
}

pub struct GameImpl<'a> {
    os: Cl<'a, dyn Os>,
    app: Cl<'a, dyn Application<'a>>,
    state: RefCell<GameState<'a>>,
}

#[injectable(scope: ApplicationComponent)]
impl<'a> GameImpl<'a> {
    #[inject]
    pub fn new(os: Cl<'a, dyn Os>, app: Cl<'a, dyn Application<'a>>) -> GameImpl<'a> {
        GameImpl {
            os,
            app,
            state: RefCell::new(GameState {
                frame_counter: 0,
                time_start: Instant::now(),
                window: None,
                t: 0.0,
            }),
        }
    }

    fn on_startup(&self) {
        self.os.message_box("foo", "bar", None);
    }
}

impl<'a> Game for GameImpl<'a> {
    fn initialize(&self) {
        self.state.borrow_mut().time_start = Instant::now();
        self.state.borrow_mut().window = Some(self.app.create_window());
        self.state.borrow().window.as_ref().unwrap().show()
    }

    fn update(&self, dt: f32) {
        let mut state = self.state.borrow_mut();
        state.frame_counter += 1;
        state.t += dt;
        if Instant::now().duration_since(state.time_start) > Duration::from_secs(1) {
            println!("fps: {}", state.frame_counter);
            state.time_start = Instant::now();
            state.frame_counter = 0;
        }
    }

    fn render(&self) {
        let state = self.state.borrow();
        let window = state.window.as_ref().unwrap();
        let t = state.t;
        let rt = window.begin();
        rt.clear(Color::rgba(
            sine_wave(t),
            sine_wave(t + 1.0 / 3.0),
            sine_wave(t + 2.0 / 3.0),
            1.0,
        ));
        rt.present()
    }
}

fn sine_wave(x: f32) -> f32 {
    0.5 + 0.5 * f32::sin(x * std::f32::consts::TAU)
}

#[component_visible]
struct GameModule {}

#[module(install_in: ApplicationComponent)]
impl GameModule {
    #[provides]
    #[into_vec]
    pub fn provide_startup_listener<'a>(game: &'a GameImpl) -> Cl<'a, dyn StartupListener> {
        Cl::Val(Box::new(move || game.on_startup()))
    }

    #[binds]
    pub fn bind_game<'a>(game: &'a GameImpl) -> Cl<'a, dyn Game> {}
}

fn main() -> Result<(), Box<dyn Error>> {
    cubit::main()
}

lockjaw::epilogue!(root debug_output);

