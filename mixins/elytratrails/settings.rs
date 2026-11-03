use crate::game::Game;
use horizon_client_render::gfx::native_render::elytratrails as et;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static CACHE: Mutex<Option<(Instant, et::Options)>> = Mutex::new(None);

impl Game<'_> {
    pub(super) fn elytratrails_options(&self) -> et::Options {
        let now = Instant::now();
        let read = || et::Options::from_settings(|key| self.native_setting(et::ID, key));
        let Ok(mut cache) = CACHE.lock() else {
            return read();
        };
        if let Some((at, options)) = *cache {
            if now.duration_since(at) < Duration::from_secs(1) {
                return options;
            }
        }
        let options = read();
        *cache = Some((now, options));
        options
    }

    pub(super) fn elytratrails_page(&self) -> Vec<String> {
        et::labels(self.elytratrails_options())
    }

    pub(super) fn elytratrails_page_click(&self, button: usize) {
        let o = self.elytratrails_options();
        let value = match button {
            0 => et::flag_text(!o.enabled),
            1 => et::flag_text(!o.own),
            2 => et::flag_text(!o.others),
            3 => o.color.next().code(),
            4 => o.length.next().code(),
            5 => o.width.next().code(),
            6 => et::flag_text(!o.keep),
            _ => return,
        };
        self.native_set_setting(et::ID, et::KEYS[button], value);
        if let Ok(mut cache) = CACHE.lock() {
            *cache = None;
        }
    }
}
