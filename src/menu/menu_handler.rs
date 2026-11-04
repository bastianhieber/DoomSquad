//! The Menu and User Interface system.
//!
//! This module manages all the non-gameplay screens, such as the Main Menu,
//! the Pause Screen, and the Game Over screen. It handles drawing buttons and text,
//! processing mouse clicks, and taking care of saving and loading the player's progress.

use font8x8::{BASIC_FONTS, UnicodeFonts};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};
use std::fs;

use crate::audio::audio_handler::Audio;
use crate::game::gamestate::Game;
use crate::render::RendererData;
use crate::game::player::Player;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;

const COL_BG_MAIN: u32 = 0x111111;
const COL_BG_GAMEOVER: u32 = 0x330000;
const COL_BG_PAUSED: u32 = 0x002244;
const COL_TITLE_RED: u32 = 0xCC0000;
const COL_TITLE_BLUE: u32 = 0x44AAFF;
const COL_BTN_NORMAL: u32 = 0x444444;
const COL_BTN_HOVER: u32 = 0xAA0000;
const COL_TEXT_WHITE: u32 = 0xFFFFFF;

/// Dictates the high-level routing in the `main.rs` loop.
pub enum AppState {
    StartScreen,
    ControlsMenu,
    Playing,
    GameOver,
    GamePaused,
    Quit,
}


//Bitton Text ist eine Variable, Die verändert werden kann innerhalb von DefBut 

/// Simple layout container for button definitions.
struct ButtonDef {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    text: String,
}

/// Holds the minimal state required for the IMGUI.
/// Since `minifb` natively handles keyboard debouncing via `KeyRepeat::No`,
/// we only need to manually track the mouse state for edge detection (clicks).
pub struct Menu {
    was_mouse_down: bool,
    waiting_for_key: Option<String>,

}

impl Menu {
    pub fn new() -> Self {
        Self {
            was_mouse_down: false,
            waiting_for_key: None,
        }
    }

    /// Renders the main menu and handles user interactions.
    pub fn update_and_draw_start_menu(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        renderer_data: &RendererData,
        audio: &mut Audio,
    ) -> AppState {
        // Clear the buffer from the previous frame
        buffer.fill(COL_BG_MAIN);

        // Fetch user inputs
        let (mx, my, clicked) = self.get_mouse_state(window);
        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);
        let click2 = window.is_key_pressed(Key::Key2, KeyRepeat::No);
        let click3 = window.is_key_pressed(Key::Key3, KeyRepeat::No);
        let click4 = window.is_key_pressed(Key::Key4, KeyRepeat::No);
        //ControlsMenu let
        

        // Render titles
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "DOOMSQUAD",
            50,
            6,
            COL_TITLE_RED,
        );
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "MAIN MENU",
            105,
            2,
            COL_TEXT_WHITE,
        );

        // Define button layouts
        let btn_start = ButtonDef {
            x: 230,
            y: 140,
            w: 340,
            h: 50,
            text: "[1] NEW GAME".to_string(),
        };
        let btn_load = ButtonDef {
            x: 230,
            y: 200,
            w: 340,
            h: 50,
            text: "[2] LOAD MAP".to_string(),
        };

        let audio_text = if !audio.is_muted {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };
        let btn_audio = ButtonDef {
            x: 230,
            y: 260,
            w: 340,
            h: 50,
            text: audio_text.to_string(),
        };

        let btn_quit = ButtonDef {
            x: 230,
            y: 380,
            w: 340,
            h: 50,
            text: "[ESC] QUIT GAME".to_string(),
        };

        let btn_controls = ButtonDef {
            x: 230,
            y: 320,
            w: 340,
            h: 50,
            text: "[4] CHANGE CONTROLS".to_string(),
        };

        // Interaction processing
        if self.draw_flat_button(buffer, &btn_start, mx, my) && clicked || click1 {
            let keep_keys = game.player.keys.clone(); // Preserve the current key bindings
            
            *game = Game::new_game(renderer_data).unwrap();
            game.player.keys = keep_keys; // Restore the preserved key bindings
            game.map_swap(renderer_data, 1); // Jump straight to map 1 (map 0 is reserved for testing)
            return AppState::Playing;
        }

        #[allow(clippy::collapsible_if)]
        if self.draw_flat_button(buffer, &btn_load, mx, my) && clicked || click2 {
            // Read saved map index from disk and safely parse it
            if let Ok(content) = fs::read_to_string("savegame.txt")
                && let Ok(index) = content.trim().parse::<usize>()
            {
                *game = Game::new_game(renderer_data).unwrap(); // ! this unwrap is intentionally accepted
                game.map_swap(renderer_data, index);
                return AppState::Playing;
            }
        }

        if self.draw_flat_button(buffer, &btn_audio, mx, my) && clicked || click3 {
            audio.set_muted(!audio.is_muted);
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_quit, mx, my) && clicked {
            return AppState::Quit;
        }

        if self.draw_flat_button(buffer, &btn_controls, mx, my) && clicked || click4 {
            return AppState::ControlsMenu;
        }

        // Keep looping in the menu until an action changes the state
        AppState::StartScreen
    }

    /// Renders the death screen and allows the user to save their current level progress.
    pub fn update_and_draw_game_over(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
    ) -> AppState {
        buffer.fill(COL_BG_GAMEOVER);

        let (mx, my, clicked) = self.get_mouse_state(window);
        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "YOU DIED",
            80,
            6,
            COL_TITLE_RED,
        );

        let btn_save = ButtonDef {
            x: 230,
            y: 220,
            w: 340,
            h: 60,
            text: "[1] SAVE MAP INDEX".to_string(),
        };
        let btn_menu = ButtonDef {
            x: 230,
            y: 300,
            w: 340,
            h: 60,
            text: "[ESC] MAIN MENU".to_string(),
        };

        if self.draw_flat_button(buffer, &btn_save, mx, my) && clicked || click1 {
            // Persist the current level index to disk so it can be loaded later
            let _ = fs::write("savegame.txt", game.map_index.to_string());
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

        AppState::GameOver
    }

    /// Renders the pause menu (triggered by pressing ESC during gameplay).
    pub fn update_and_draw_game_paused(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        audio: &mut Audio,
    ) -> AppState {
        buffer.fill(COL_BG_PAUSED);

        let (mx, my, clicked) = self.get_mouse_state(window);

        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);
        let click2 = window.is_key_pressed(Key::Key2, KeyRepeat::No);
        let click3 = window.is_key_pressed(Key::Key3, KeyRepeat::No);

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "GAME PAUSED",
            60,
            6,
            COL_TITLE_BLUE,
        );

        let audio_text = if !audio.is_muted {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };

        let btn_resume = ButtonDef {
            x: 230,
            y: 150,
            w: 340,
            h: 50,
            text: "[1] RESUME GAME".to_string(),
        };
        let btn_save = ButtonDef {
            x: 230,
            y: 210,
            w: 340,
            h: 50,
            text: "[2] SAVE & EXIT".to_string(),
        };
        let btn_audio = ButtonDef {
            x: 230,
            y: 270,
            w: 340,
            h: 50,
            text: audio_text.to_string(),
        };
        let btn_menu = ButtonDef {
            x: 230,
            y: 330,
            w: 340,
            h: 50,
            text: "[ESC] MAIN MENU".to_string(),
        };

        

        if self.draw_flat_button(buffer, &btn_resume, mx, my) && clicked || click1 {
            // Simply return to the Playing state without touching the game object to resume seamlessly
            return AppState::Playing;
        }

        if self.draw_flat_button(buffer, &btn_save, mx, my) && clicked || click2 {
            let _ = std::fs::write("savegame.txt", game.map_index.to_string());
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_audio, mx, my) && clicked || click3 {
            audio.set_muted(!audio.is_muted);
            return AppState::GamePaused;
        }

        if self.draw_flat_button(buffer, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

    

        AppState::GamePaused
    }

    /// Fetches mouse coordinates and performs edge detection (debouncing) for left-clicks.
    fn get_mouse_state(&mut self, window: &Window) -> (usize, usize, bool) {
        // Clamp prevents out-of-bounds coordinates if the mouse leaves the window frame
        let mouse_pos = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let is_mouse_down = window.get_mouse_down(MouseButton::Left);

        let clicked = is_mouse_down && !self.was_mouse_down;
        self.was_mouse_down = is_mouse_down;

        (mouse_pos.0 as usize, mouse_pos.1 as usize, clicked)
    }

    /// Draws a rectangular button and its text onto the buffer.
    /// Returns `true` if the mouse cursor is currently hovering over it.
    fn draw_flat_button(&self, buffer: &mut [u32], btn: &ButtonDef, mx: usize, my: usize) -> bool {
        // Simple check for hover detection
        let hover = mx >= btn.x && mx <= (btn.x + btn.w) && my >= btn.y && my <= (btn.y + btn.h);
        let bg_color = if hover { COL_BTN_HOVER } else { COL_BTN_NORMAL };

        // Rasterize the rectangle into the 1D pixel buffer
        for row in btn.y..(btn.y + btn.h) {
            for col in btn.x..(btn.x + btn.w) {
                // Safety check to prevent panic from out-of-bounds memory writes
                if row < SCREEN_HEIGHT && col < SCREEN_WIDTH {
                    buffer[row * SCREEN_WIDTH + col] = bg_color;
                }
            }
        }

        // Center the text horizontally and vertically inside the button bounds
        let text_width = btn.text.len() * 8 * 2;
        let text_x = btn.x + btn.w.saturating_sub(text_width) / 2;
        let text_y = btn.y + btn.h.saturating_sub(16) / 2;

        Self::draw_text(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &btn.text,
            text_x,
            text_y,
            2,
            COL_TEXT_WHITE,
        );

        hover
    }

    /// Helper to align a given text string exactly in the horizontal center of the screen.
    fn draw_text_centered(
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        text: &str,
        y: usize,
        scale: usize,
        color: u32,
    ) {
        // Each character in the font is 8 pixels wide, multiplied by the scaling factor
        let text_width = text.len() * 8 * scale;
        let x = screen_width.saturating_sub(text_width) / 2;

        Self::draw_text(
            buffer,
            screen_width,
            screen_height,
            text,
            x,
            y,
            scale,
            color,
        );
    }

    /// Software text rasterizer using the `font8x8` library.
    /// It translates strings into raw pixel data and pushes them to the 1D buffer.
    #[allow(clippy::too_many_arguments)]
    fn draw_text(
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        text: &str,
        start_x: usize,
        start_y: usize,
        scale: usize,
        color: u32,
    ) {
        let mut cursor_x = start_x;

        for c in text.chars() {
            if let Some(glyph) = BASIC_FONTS.get(c) {
                #[allow(clippy::needless_range_loop)]
                for y in 0..8 {
                    for x in 0..8 {
                        // Bitwise check: if the specific bit in the glyph byte is a 1, a pixel needs to be drawn
                        if (glyph[y] & (1 << x)) != 0 {
                            // Nearest-neighbor scaling (draws larger 2D blocks for each active pixel based on the `scale` factor)
                            for dy in 0..scale {
                                for dx in 0..scale {
                                    let px = cursor_x + x * scale + dx;
                                    let py = start_y + y * scale + dy;

                                    if px < screen_width && py < screen_height {
                                        // Flatten 2D coordinate into a 1D array index
                                        buffer[py * screen_width + px] = color;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Advance the cursor by the width of the character to prepare for the next letter
            cursor_x += 8 * scale;
        }
    }



//------------------------------------------------------------------------------------------------------------------------------
   
// Renders the controls menu and handles user interactions.
    pub fn update_and_draw_controls_menu(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        renderer_data: &RendererData,
        audio: &mut Audio,
    ) -> AppState {
        // Clear the buffer from the previous frame
        buffer.fill(COL_BG_MAIN);

        // Fetch user inputs
        let (mx, my, clicked) = self.get_mouse_state(window);
       
        let click4 = window.is_key_pressed(Key::Key4, KeyRepeat::No);

         

        // Render titles
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "Controls",
            15,
            4,
            COL_TEXT_WHITE,
        );
        

        // Define button layouts

    //WAITING LETS
        let waiting_forward = self.waiting_for_key.as_deref() == Some("forward");
        let waiting_backward = self.waiting_for_key.as_deref() == Some("backward");
        let waiting_left = self.waiting_for_key.as_deref() == Some("left");
        let waiting_right = self.waiting_for_key.as_deref() == Some("right");
        let waiting_sprint = self.waiting_for_key.as_deref() == Some("sprint");
        let waiting_jump = self.waiting_for_key.as_deref() == Some("jump");
        let waiting_crouch = self.waiting_for_key.as_deref() == Some("crouch");
        let waiting_rocket = self.waiting_for_key.as_deref() == Some("rocket");
        let waiting_interact = self.waiting_for_key.as_deref() == Some("interact");
        let waiting_aim_up = self.waiting_for_key.as_deref() == Some("aim_up");
        let waiting_aim_down = self.waiting_for_key.as_deref() == Some("aim_down");
        let waiting_turn_left = self.waiting_for_key.as_deref() == Some("turn_left");
        let waiting_turn_right = self.waiting_for_key.as_deref() == Some("turn_right");
        let waiting_fine_aim = self.waiting_for_key.as_deref() == Some("fine_aim");
        let waiting_fire_bullet = self.waiting_for_key.as_deref() == Some("fire_bullet");
        let waiting_fire_arrow = self.waiting_for_key.as_deref() == Some("fire_arrow");
        let waiting_godmode = self.waiting_for_key.as_deref() == Some("godmode");
        let waiting_lock_all = self.waiting_for_key.as_deref() == Some("lock_all");


let btn_forward = ButtonDef {
    x: 30,
    y: 50,
    w: 340,
    h: 30,
    text: if waiting_forward {
        "FORWARD: PRESS A KEY".to_string()
    } else {
        format!("FORWARD: {:?}", game.player.keys.forward_key)
    },
};

let btn_backward = ButtonDef {
    x: 30,
    y: 90,
    w: 340,
    h: 30,
    text: if waiting_backward {
        "BACKWARD: PRESS A KEY".to_string()
    } else {
        format!("BACKWARD: {:?}", game.player.keys.backward_key)
    },
};

let btn_left = ButtonDef {
    x: 30,
    y: 130,
    w: 340,
    h: 30,
    text: if waiting_left {
        "LEFT: PRESS A KEY".to_string()
    } else {
        format!("LEFT: {:?}", game.player.keys.left_key)
    },
};

let btn_right = ButtonDef {
    x: 30,
    y: 170,
    w: 340,
    h: 30,
    text: if waiting_right {
        "RIGHT: PRESS A KEY".to_string()
    } else {
        format!("RIGHT: {:?}", game.player.keys.right_key)
    },
};

let btn_sprint = ButtonDef {
    x: 30,
    y: 210,
    w: 340,
    h: 30,
    text: if waiting_sprint {
        "SPRINT: PRESS A KEY".to_string()
    } else {
        format!("SPRINT: {:?}", game.player.keys.sprint_key)
    },
};

let btn_jump = ButtonDef {
    x: 30,
    y: 250,
    w: 340,
    h: 30,
    text: if waiting_jump {
        "JUMP: PRESS A KEY".to_string()
    } else {
        format!("JUMP: {:?}", game.player.keys.jump_key)
    },
};

let btn_crouch = ButtonDef {
    x: 30,
    y: 290,
    w: 340,
    h: 30,
    text: if waiting_crouch {
        "CROUCH: PRESS A KEY".to_string()
    } else {
        format!("CROUCH: {:?}", game.player.keys.crouch_key)
    },
};

let btn_rocket = ButtonDef {
    x: 30,
    y: 330,
    w: 340,
    h: 30,
    text: if waiting_rocket {
        "ROCKET: PRESS A KEY".to_string()
    } else {
        format!("ROCKET: {:?}", game.player.keys.rocket_jump_key)
    },
};

let btn_interact = ButtonDef {
    x: 30,
    y: 370,
    w: 340,
    h: 30,
    text: if waiting_interact {
        "INTERACT: PRESS A KEY".to_string()
    } else {
        format!("INTERACT: {:?}", game.player.keys.interact_key)
    },
};

let btn_aim_up = ButtonDef {
    x: 400,
    y: 50,
    w: 340,
    h: 30,
    text: if waiting_aim_up {
        "AIM UP: PRESS A KEY".to_string()
    } else {
        format!("AIM UP: {:?}", game.player.keys.move_aim_up_key)
    },
};

let btn_aim_down = ButtonDef {
    x: 400,
    y: 90,
    w: 340,
    h: 30,
    text: if waiting_aim_down {
        "AIM DOWN: PRESS A KEY".to_string()
    } else {
        format!("AIM DOWN: {:?}", game.player.keys.move_aim_down_key)
    },
};

let btn_turn_left = ButtonDef {
    x: 400,
    y: 130,
    w: 340,
    h: 30,
    text: if waiting_turn_left {
        "TURN LEFT: PRESS A KEY".to_string()
    } else {
        format!("TURN LEFT: {:?}", game.player.keys.turn_left_key)
    },
};

let btn_turn_right = ButtonDef {
    x: 400,
    y: 170,
    w: 340,
    h: 30,
    text: if waiting_turn_right {
        "TURN RIGHT: PRESS A KEY".to_string()
    } else {
        format!("TURN RIGHT: {:?}", game.player.keys.turn_right_key)
    },
};

let btn_fine_aim = ButtonDef {
    x: 400,
    y: 210,
    w: 340,
    h: 30,
    text: if waiting_fine_aim {
        "FINE AIM: PRESS A KEY".to_string()
    } else {
        format!("FINE AIM: {:?}", game.player.keys.fine_aiming_key)
    },
};

let btn_fire_bullet = ButtonDef {
    x: 400,
    y: 250,
    w: 340,
    h: 30,
    text: if waiting_fire_bullet {
        "FIRE BULLET: PRESS A KEY".to_string()
    } else {
        format!("FIRE BULLET: {:?}", game.player.keys.fire_bullet_key)
    },
};

let btn_fire_arrow = ButtonDef {
    x: 400,
    y: 290,
    w: 340,
    h: 30,
    text: if waiting_fire_arrow {
        "FIRE ARROW: PRESS A KEY".to_string()
    } else {
        format!("FIRE ARROW: {:?}", game.player.keys.fire_arrow_key)
    },
};

let btn_godmode = ButtonDef {
    x: 400,
    y: 330,
    w: 340,
    h: 30,
    text: if waiting_godmode {
        "GODMODE: PRESS A KEY".to_string()
    } else {
        format!("GODMODE: {:?}", game.player.keys.god_mode_key)
    },
};

let btn_lock_all = ButtonDef {
    x: 400,
    y: 370,
    w: 340,
    h: 30,
    text: if waiting_lock_all {
        "LOCK ALL: PRESS A KEY".to_string()
    } else {
        format!("LOCK ALL: {:?}", game.player.keys.lock_all_key)
    },
};

let btn_menu_return = ButtonDef {
    x: 30,
    y: 410,
    w: 340,
    h: 30,
    text: "BACK TO MAIN MENU".to_string(),
};




if self.draw_flat_button(buffer, &btn_forward, mx, my) && clicked {
    self.waiting_for_key = Some("forward".to_string());
}else if self.draw_flat_button(buffer, &btn_backward, mx, my) && clicked {
    self.waiting_for_key = Some("backward".to_string());
} else if self.draw_flat_button(buffer, &btn_left, mx, my) && clicked {
    self.waiting_for_key = Some("left".to_string());
} else if self.draw_flat_button(buffer, &btn_right, mx, my) && clicked {
    self.waiting_for_key = Some("right".to_string());
} else if self.draw_flat_button(buffer, &btn_sprint, mx, my) && clicked {
    self.waiting_for_key = Some("sprint".to_string());
} else if self.draw_flat_button(buffer, &btn_jump, mx, my) && clicked {
    self.waiting_for_key = Some("jump".to_string());
} else if self.draw_flat_button(buffer, &btn_crouch, mx, my) && clicked {
    self.waiting_for_key = Some("crouch".to_string());
} else if self.draw_flat_button(buffer, &btn_rocket, mx, my) && clicked {
    self.waiting_for_key = Some("rocket".to_string());
} else if self.draw_flat_button(buffer, &btn_interact, mx, my) && clicked {
    self.waiting_for_key = Some("interact".to_string());
} else if self.draw_flat_button(buffer, &btn_aim_up, mx, my) && clicked {
    self.waiting_for_key = Some("aim_up".to_string());
} else if self.draw_flat_button(buffer, &btn_aim_down, mx, my) && clicked {
    self.waiting_for_key = Some("aim_down".to_string());
} else if self.draw_flat_button(buffer, &btn_turn_left, mx, my) && clicked {
    self.waiting_for_key = Some("turn_left".to_string());
} else if self.draw_flat_button(buffer, &btn_turn_right, mx, my) && clicked {
    self.waiting_for_key = Some("turn_right".to_string());
} else if self.draw_flat_button(buffer, &btn_fine_aim, mx, my) && clicked {
    self.waiting_for_key = Some("fine_aim".to_string());
} else if self.draw_flat_button(buffer, &btn_fire_bullet, mx, my) && clicked {
    self.waiting_for_key = Some("fire_bullet".to_string());
} else if self.draw_flat_button(buffer, &btn_fire_arrow, mx, my) && clicked {
    self.waiting_for_key = Some("fire_arrow".to_string());
} else if self.draw_flat_button(buffer, &btn_godmode, mx, my) && clicked {
    self.waiting_for_key = Some("godmode".to_string());
} else if self.draw_flat_button(buffer, &btn_lock_all, mx, my) && clicked {
    self.waiting_for_key = Some("lock_all".to_string());
}






if waiting_forward {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.forward_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_backward {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.backward_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_left {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.left_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_right {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.right_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_sprint {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.sprint_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_jump {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.jump_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_crouch {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.crouch_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_rocket {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.rocket_jump_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    } 
}else if waiting_interact {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.interact_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_aim_up {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.move_aim_up_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_aim_down {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.move_aim_down_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_turn_left {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.turn_left_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_turn_right {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.turn_right_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_fine_aim {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.fine_aiming_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_fire_bullet {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.fire_bullet_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_fire_arrow {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.fire_arrow_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_godmode {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.god_mode_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}else if waiting_lock_all {
    if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first().copied() {
        game.player.keys.lock_all_key = new_key;
        self.waiting_for_key = None;
        game.player.keys.save();
    }
}

if self.draw_flat_button(buffer, &btn_menu_return, mx, my) && clicked {
    self.waiting_for_key = None;
    return AppState::StartScreen;
}

AppState::ControlsMenu
    }
}


