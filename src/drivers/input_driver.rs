use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct InputDriver {
    event_pump: EventPump
}

impl InputDriver {

    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let event_pump = sdl_context.event_pump().unwrap();

        InputDriver {
            event_pump
        }
    } 

    pub fn update(&mut self) -> Result<[bool; 16], ()> {

        let keyboard_mapping:[Keycode; 16] = [
            Keycode::Num1,  Keycode::Num2,  Keycode::Num3,  Keycode::Num4,
            Keycode::Q,     Keycode::W,     Keycode::E,     Keycode::R,
            Keycode::A,     Keycode::S,     Keycode::D,     Keycode::F,
            Keycode::Z,     Keycode::X,     Keycode::C,     Keycode::V
        ];

        for event in self.event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                return Err(());
            };
            if let Event::KeyDown { keycode: Some(Keycode::Escape), .. } = event {
                return Err(());
            };
        }

        let keys: Vec<Keycode> = self.event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let keymap = keyboard_mapping.map(|f: Keycode| keys.contains(&f));
        Ok(keymap)
    }

}