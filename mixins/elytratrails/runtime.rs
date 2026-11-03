use std::collections::{BTreeMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub const ID: &str = "elytratrails";
#[allow(dead_code)]
pub const NAME: &str = "ElytraTrails";
#[allow(dead_code)]
pub const VERSION: &str = "0.1.0";
#[allow(dead_code)]
pub const DESCRIPTION: &str = "Wing trails behind a player who glides with elytra.";

pub const LIFETIME: Duration = Duration::from_millis(900);
pub const KEEP: Duration = Duration::from_millis(1500);
pub const VISIBLE_FOR: Duration = Duration::from_millis(250);
pub const PRUNE_EVERY: Duration = Duration::from_secs(1);
pub const MIN_STEP: f64 = 0.05;
pub const MAX_POINTS: usize = 512;
pub const HALF_WIDTH: f64 = 0.06;
pub const HEIGHT: f64 = 0.12;
pub const ROOT_UP: f64 = 1.25;
pub const TIP_BACK: f64 = 0.9;
pub const TIP_SIDE: f64 = 0.95;
pub const TIP_SMOOTH: f64 = 0.35;
pub const ALPHA: f32 = 0.9;
pub const SYNC_CHANNEL: &str = "elytratrails:color";
pub const SYNC_EVERY: Duration = Duration::from_secs(5);

pub type Vec3d = [f64; 3];

pub const KEYS: [&str; 7] = ["enabled", "own", "others", "color", "length", "width", "keep"];

fn cycle<T: Copy + PartialEq>(all: &[T], at: T) -> T {
    let i = all.iter().position(|c| *c == at).unwrap_or(0);
    all[(i + 1) % all.len()]
}

fn parse<T: Copy>(all: &[T], code: fn(T) -> &'static str, s: &str) -> Option<T> {
    all.iter().copied().find(|&v| code(v) == s)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Ice,
    Gold,
    Rose,
    Violet,
    White,
}

impl Color {
    pub const ALL: [Color; 5] = [Color::Ice, Color::Gold, Color::Rose, Color::Violet, Color::White];

    pub fn for_code(code: &str) -> Self {
        parse(&Self::ALL, Self::code, code).unwrap_or(Color::Ice)
    }

    pub fn code(self) -> &'static str {
        match self {
            Color::Ice => "ice",
            Color::Gold => "gold",
            Color::Rose => "rose",
            Color::Violet => "violet",
            Color::White => "white",
        }
    }

    pub fn tint(self) -> u32 {
        match self {
            Color::Ice => 0xB8_F0_FF,
            Color::Gold => 0xFF_D3_6B,
            Color::Rose => 0xFF_8F_B1,
            Color::Violet => 0xC9_A2_FF,
            Color::White => 0xFF_FF_FF,
        }
    }

    pub fn next(self) -> Self {
        cycle(&Self::ALL, self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Length {
    Short,
    Normal,
    Long,
}

impl Length {
    pub const ALL: [Length; 3] = [Length::Short, Length::Normal, Length::Long];

    pub fn for_code(code: &str) -> Self {
        parse(&Self::ALL, Self::code, code).unwrap_or(Length::Normal)
    }

    pub fn code(self) -> &'static str {
        match self {
            Length::Short => "short",
            Length::Normal => "normal",
            Length::Long => "long",
        }
    }

    pub fn duration(self) -> Duration {
        match self {
            Length::Short => Duration::from_millis(500),
            Length::Normal => LIFETIME,
            Length::Long => KEEP,
        }
    }

    pub fn next(self) -> Self {
        cycle(&Self::ALL, self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    Thin,
    Normal,
    Wide,
}

impl Width {
    pub const ALL: [Width; 3] = [Width::Thin, Width::Normal, Width::Wide];

    pub fn for_code(code: &str) -> Self {
        parse(&Self::ALL, Self::code, code).unwrap_or(Width::Normal)
    }

    pub fn code(self) -> &'static str {
        match self {
            Width::Thin => "thin",
            Width::Normal => "normal",
            Width::Wide => "wide",
        }
    }

    pub fn factor(self) -> f64 {
        match self {
            Width::Thin => 0.5,
            Width::Normal => 1.0,
            Width::Wide => 1.6,
        }
    }

    pub fn next(self) -> Self {
        cycle(&Self::ALL, self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub enabled: bool,
    pub own: bool,
    pub others: bool,
    pub color: Color,
    pub length: Length,
    pub width: Width,
    pub keep: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            enabled: true,
            own: true,
            others: true,
            color: Color::Ice,
            length: Length::Normal,
            width: Width::Normal,
            keep: true,
        }
    }
}

impl Options {
    pub fn from_settings(get: impl Fn(&str) -> Option<String>) -> Self {
        let on = |key: &str| !matches!(get(key).as_deref(), Some("off"));
        Self {
            enabled: on(KEYS[0]),
            own: on(KEYS[1]),
            others: on(KEYS[2]),
            color: Color::for_code(get(KEYS[3]).as_deref().unwrap_or("")),
            length: Length::for_code(get(KEYS[4]).as_deref().unwrap_or("")),
            width: Width::for_code(get(KEYS[5]).as_deref().unwrap_or("")),
            keep: on(KEYS[6]),
        }
    }

    pub fn shows(&self, local: bool) -> bool {
        self.enabled && if local { self.own } else { self.others }
    }
}

pub fn flag_text(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

pub fn labels(o: Options) -> Vec<String> {
    vec![
        format!("Trails: {}", flag_text(o.enabled)),
        format!("Your trail: {}", flag_text(o.own)),
        format!("Other players' trails: {}", flag_text(o.others)),
        format!("Color: {}", o.color.code()),
        format!("Length: {}", o.length.code()),
        format!("Width: {}", o.width.code()),
        format!("Keep trails out of view: {}", flag_text(o.keep)),
    ]
}

pub fn wing_tips(feet: Vec3d, body_yaw: f32) -> [Vec3d; 2] {
    let yaw = f64::from(body_yaw).to_radians();
    let forward = [-yaw.sin(), 0.0, yaw.cos()];
    let right = [-yaw.cos(), 0.0, -yaw.sin()];
    let root = [feet[0], feet[1] + ROOT_UP, feet[2]];
    let tip = |side: f64| {
        [0, 1, 2].map(|i| root[i] - forward[i] * TIP_BACK + right[i] * side * TIP_SIDE)
    };
    [tip(-1.0), tip(1.0)]
}

pub fn to_local(point: Vec3d, feet: Vec3d, body_yaw: f32) -> [f32; 3] {
    let yaw = f64::from(body_yaw).to_radians();
    let (sin, cos) = (yaw.sin(), yaw.cos());
    let (x, y, z) = (point[0] - feet[0], point[1] - feet[1], point[2] - feet[2]);
    [(x * cos + z * sin) as f32, y as f32, (-x * sin + z * cos) as f32]
}

pub fn sheets(a: Vec3d, b: Vec3d, scale: f64) -> [[Vec3d; 4]; 2] {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    let len = (dx * dx + dz * dz).sqrt();
    let (px, pz) = if len > 1e-6 {
        (-dz / len * HALF_WIDTH * scale, dx / len * HALF_WIDTH * scale)
    } else {
        (HALF_WIDTH * scale, 0.0)
    };
    let up = HEIGHT * scale;
    let flat = [
        [a[0] + px, a[1], a[2] + pz],
        [b[0] + px, b[1], b[2] + pz],
        [b[0] - px, b[1], b[2] - pz],
        [a[0] - px, a[1], a[2] - pz],
    ];
    let standing = [
        [a[0], a[1] + up, a[2]],
        [b[0], b[1] + up, b[2]],
        [b[0], b[1], b[2]],
        [a[0], a[1], a[2]],
    ];
    [flat, standing]
}

#[derive(Default)]
struct Wing {
    points: VecDeque<(Vec3d, Instant)>,
    smooth: Option<Vec3d>,
}

#[derive(Default)]
struct Trail {
    seen: Option<Instant>,
    local: bool,
    name: String,
    wings: [Wing; 2],
}

struct Store {
    trails: BTreeMap<u32, Trail>,
    pruned: Option<Instant>,
}

static STORE: Mutex<Store> = Mutex::new(Store {
    trails: BTreeMap::new(),
    pruned: None,
});

static OPTIONS: Mutex<Option<Options>> = Mutex::new(None);

pub fn set_options(options: Options) {
    if let Ok(mut current) = OPTIONS.lock() {
        *current = Some(options);
    }
}

pub fn current_options() -> Options {
    OPTIONS.lock().ok().and_then(|current| *current).unwrap_or_default()
}

fn dist_sq(a: Vec3d, b: Vec3d) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

pub struct Viewer<'a> {
    pub local: bool,
    pub name: &'a str,
}

#[cfg(test)]
pub fn record(id: u32, feet: Vec3d, body_yaw: f32, gliding: bool, now: Instant) {
    let viewer = Viewer { local: true, name: "test" };
    record_at(id, feet, body_yaw, gliding, wing_tips(feet, body_yaw), now, viewer);
}

pub fn record_at(
    id: u32,
    feet: Vec3d,
    body_yaw: f32,
    gliding: bool,
    tips: [Vec3d; 2],
    now: Instant,
    viewer: Viewer<'_>,
) {
    let Ok(mut store) = STORE.lock() else {
        return;
    };
    let trail = store.trails.entry(id).or_default();
    trail.seen = Some(now);
    trail.local = viewer.local;
    if trail.name != viewer.name {
        trail.name.clear();
        trail.name.push_str(viewer.name);
    }
    for (wing, tip) in trail.wings.iter_mut().zip(tips) {
        while wing
            .points
            .front()
            .is_some_and(|(_, born)| now.duration_since(*born) >= KEEP)
        {
            wing.points.pop_front();
        }
        if !gliding {
            wing.smooth = None;
            continue;
        }
        let point = match wing.smooth {
            Some(prev) => [0, 1, 2].map(|i| prev[i] + (tip[i] - prev[i]) * TIP_SMOOTH),
            None => tip,
        };
        wing.smooth = Some(point);
        let far_enough = wing
            .points
            .back()
            .is_none_or(|(last, _)| dist_sq(*last, point) >= MIN_STEP * MIN_STEP);
        if far_enough {
            wing.points.push_back((point, now));
            if wing.points.len() > MAX_POINTS {
                wing.points.pop_front();
            }
        }
    }
}

fn prune(store: &mut Store, now: Instant) {
    if store
        .pruned
        .is_some_and(|at| now.duration_since(at) < PRUNE_EVERY)
    {
        return;
    }
    store
        .trails
        .retain(|_, t| t.seen.is_some_and(|s| now.duration_since(s) < KEEP));
    store.pruned = Some(now);
}

fn push_segments(trail: &Trail, now: Instant, lifetime: Duration, out: &mut Vec<(Vec3d, Vec3d, f64)>) {
    let life = lifetime.as_secs_f64();
    for wing in &trail.wings {
        for ((p, born), (q, _)) in wing.points.iter().zip(wing.points.iter().skip(1)) {
            let age = now.duration_since(*born);
            if age >= lifetime {
                continue;
            }
            out.push((*p, *q, 1.0 - age.as_secs_f64() / life));
        }
    }
}

#[cfg(test)]
pub fn segments(id: u32, now: Instant, lifetime: Duration) -> Vec<(Vec3d, Vec3d, f64)> {
    let mut out = Vec::new();
    if let Ok(store) = STORE.lock() {
        if let Some(trail) = store.trails.get(&id) {
            push_segments(trail, now, lifetime, &mut out);
        }
    }
    out
}

pub struct Ribbon {
    pub a: Vec3d,
    pub b: Vec3d,
    pub width: f64,
    pub alpha: f32,
    pub tint: u32,
}

pub fn ribbons(now: Instant, options: Options) -> Vec<Ribbon> {
    let Ok(mut store) = STORE.lock() else {
        return Vec::new();
    };
    prune(&mut store, now);
    let life = options.length.duration();
    let width = options.width.factor();
    let mut out = Vec::new();
    let mut segments = Vec::new();
    for trail in store.trails.values() {
        if !options.shows(trail.local) {
            continue;
        }
        let in_view = trail.seen.is_some_and(|s| now.duration_since(s) < VISIBLE_FOR);
        if !options.keep && !in_view {
            continue;
        }
        let tint = if trail.local {
            options.color.tint()
        } else {
            remote_color(&trail.name).unwrap_or(Color::Ice).tint()
        };
        segments.clear();
        push_segments(trail, now, life, &mut segments);
        for (a, b, fade) in segments.drain(..) {
            out.push(Ribbon {
                a,
                b,
                width: width * fade,
                alpha: fade as f32 * ALPHA,
                tint,
            });
        }
    }
    out
}

static REMOTE_COLORS: Mutex<BTreeMap<String, Color>> = Mutex::new(BTreeMap::new());

pub fn set_remote_color(name: &str, color: Color) {
    if let Ok(mut map) = REMOTE_COLORS.lock() {
        map.insert(name.to_owned(), color);
    }
}

pub fn remote_color(name: &str) -> Option<Color> {
    REMOTE_COLORS.lock().ok()?.get(name).copied()
}

static LAST_SENT: Mutex<Option<Instant>> = Mutex::new(None);

pub fn sync_due(now: Instant) -> bool {
    let Ok(mut last) = LAST_SENT.lock() else {
        return false;
    };
    if last.is_some_and(|at| now.duration_since(at) < SYNC_EVERY) {
        return false;
    }
    *last = Some(now);
    true
}

pub fn color_from_code_bytes(data: &[u8]) -> Option<Color> {
    let code = std::str::from_utf8(data).ok()?;
    parse(&Color::ALL, Color::code, code)
}

pub fn encode_remote(name: &str, color: Color) -> Vec<u8> {
    format!("{name}|{}", color.code()).into_bytes()
}

pub fn decode_remote(data: &[u8]) -> Option<(String, Color)> {
    let text = std::str::from_utf8(data).ok()?;
    let (name, code) = text.split_once('|')?;
    Some((name.to_owned(), parse(&Color::ALL, Color::code, code)?))
}

struct ServerPlayer {
    name: String,
    color: Option<Color>,
}

static SERVER_PLAYERS: Mutex<BTreeMap<u128, ServerPlayer>> = Mutex::new(BTreeMap::new());

pub fn server_joined(id: u128, name: &str) -> Vec<Vec<u8>> {
    let Ok(mut map) = SERVER_PLAYERS.lock() else {
        return Vec::new();
    };
    map.insert(id, ServerPlayer { name: name.to_owned(), color: None });
    map.iter()
        .filter(|(other, _)| **other != id)
        .filter_map(|(_, p)| p.color.map(|c| encode_remote(&p.name, c)))
        .collect()
}

pub fn server_left(id: u128) {
    if let Ok(mut map) = SERVER_PLAYERS.lock() {
        map.remove(&id);
    }
}

pub fn server_colour(id: u128, data: &[u8]) -> Option<(Vec<u128>, Vec<u8>)> {
    let color = color_from_code_bytes(data)?;
    let mut map = SERVER_PLAYERS.lock().ok()?;
    let player = map.get_mut(&id)?;
    if player.color == Some(color) {
        return None;
    }
    player.color = Some(color);
    let message = encode_remote(&player.name, color);
    let to = map.keys().copied().filter(|other| *other != id).collect();
    Some((to, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn facing_south_the_wings_trail_behind_and_out_to_each_side() {
        let [left, right] = wing_tips([0.0, 0.0, 0.0], 0.0);
        assert!(close(left[2], -TIP_BACK) && close(right[2], -TIP_BACK));
        assert!(close(right[0], -TIP_SIDE));
        assert!(close(left[0], TIP_SIDE));
        assert!(close(left[1], ROOT_UP));
    }

    #[test]
    fn the_wings_turn_with_the_body() {
        let [_, right] = wing_tips([0.0, 0.0, 0.0], 90.0);
        assert!(close(right[0], TIP_BACK));
        assert!(close(right[2], -TIP_SIDE));
    }

    #[test]
    fn a_point_straight_ahead_is_plus_z_in_the_model_frame() {
        let yaw = 30.0_f64.to_radians();
        let ahead = [-yaw.sin(), 0.0, yaw.cos()];
        let local = to_local(ahead, [0.0, 0.0, 0.0], 30.0);
        assert!((local[0]).abs() < 1e-5, "x was {}", local[0]);
        assert!((local[2] - 1.0).abs() < 1e-5, "z was {}", local[2]);
    }

    #[test]
    fn a_point_to_the_players_left_is_plus_x() {
        let local = to_local([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.0);
        assert!((local[0] - 1.0).abs() < 1e-5);
        assert!(local[2].abs() < 1e-5);
    }

    #[test]
    fn a_segment_is_two_ribbons_one_flat_and_one_standing() {
        let sheets = sheets([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0);
        assert_eq!(sheets.len(), 2);
        assert!(sheets[1][0][1] > sheets[0][0][1]);
    }

    #[test]
    fn a_faded_segment_is_narrower() {
        let full = sheets([0.0; 3], [1.0, 0.0, 0.0], 1.0);
        let half = sheets([0.0; 3], [1.0, 0.0, 0.0], 0.5);
        let width = |s: &[[f64; 3]; 4]| (s[0][2] - s[3][2]).abs();
        assert!((width(&half[0]) * 2.0 - width(&full[0])).abs() < 1e-6);
    }

    #[test]
    fn nothing_is_drawn_until_the_player_glides() {
        let now = Instant::now();
        record(9001, [0.0, 64.0, 0.0], 0.0, false, now);
        record(9001, [1.0, 64.0, 0.0], 0.0, false, now);
        assert!(segments(9001, now, LIFETIME).is_empty());
    }

    #[test]
    fn a_gliding_player_leaves_a_trail_that_fades_out() {
        let t0 = Instant::now();
        for step in 0..4 {
            let x = step as f64 * 0.5;
            record(9002, [x, 64.0, 0.0], 0.0, true, t0);
        }
        let fresh = segments(9002, t0, LIFETIME);
        assert_eq!(fresh.len(), 6);
        assert!(fresh.iter().all(|(_, _, s)| (*s - 1.0).abs() < 1e-6));
        let half_way = segments(9002, t0 + LIFETIME / 2, LIFETIME);
        assert!(half_way.iter().all(|(_, _, s)| (*s - 0.5).abs() < 0.01));
        assert!(segments(9002, t0 + LIFETIME, LIFETIME).is_empty());
    }

    #[test]
    fn a_player_that_stops_moving_adds_no_points() {
        let t0 = Instant::now();
        record(9003, [0.0, 64.0, 0.0], 0.0, true, t0);
        record(9003, [0.0, 64.0, 0.0], 0.0, true, t0);
        assert!(segments(9003, t0, LIFETIME).is_empty());
    }

    #[test]
    fn a_shorter_length_hides_points_the_store_still_keeps() {
        let t0 = Instant::now();
        for step in 0..4 {
            record(9004, [step as f64 * 0.5, 64.0, 0.0], 0.0, true, t0);
        }
        let at = t0 + Length::Short.duration();
        assert!(segments(9004, at, Length::Short.duration()).is_empty());
        assert_eq!(segments(9004, at, Length::Long.duration()).len(), 6);
    }

    #[test]
    fn every_length_fits_in_what_the_store_keeps() {
        for length in Length::ALL {
            assert!(length.duration() <= KEEP);
        }
        assert_eq!(Length::Normal.duration(), LIFETIME);
    }

    #[test]
    fn settings_default_to_on_and_read_off() {
        let all = Options::from_settings(|_| None);
        assert_eq!(all, Options::default());
        let some = Options::from_settings(|key| (key == "own").then(|| "off".to_owned()));
        assert!(!some.own);
        assert!(some.enabled && some.others && some.keep);
        let no_keep = Options::from_settings(|key| (key == "keep").then(|| "off".to_owned()));
        assert!(!no_keep.keep);
    }

    #[test]
    fn own_and_others_follow_the_master_switch() {
        let on = Options::default();
        assert!(on.shows(true) && on.shows(false));
        let no_own = Options { own: false, ..on };
        assert!(!no_own.shows(true) && no_own.shows(false));
        let off = Options { enabled: false, ..on };
        assert!(!off.shows(true) && !off.shows(false));
    }

    #[test]
    fn choices_cycle_and_round_trip_their_codes() {
        for c in Color::ALL {
            assert_eq!(Color::for_code(c.code()), c);
        }
        let mut c = Color::Ice;
        for _ in 0..Color::ALL.len() {
            c = c.next();
        }
        assert_eq!(c, Color::Ice);
        assert_eq!(Color::for_code("nonsense"), Color::Ice);
        for l in Length::ALL {
            assert_eq!(Length::for_code(l.code()), l);
        }
        assert_eq!(Length::Long.next(), Length::Short);
        for w in Width::ALL {
            assert_eq!(Width::for_code(w.code()), w);
        }
        assert_eq!(Width::Wide.next(), Width::Thin);
    }

    #[test]
    fn labels_follow_the_settings_in_button_order() {
        let all = labels(Options::default());
        assert_eq!(all.len(), KEYS.len());
        assert_eq!(all[0], "Trails: on");
        assert_eq!(all[2], "Other players' trails: on");
        assert_eq!(all[3], "Color: ice");
        assert_eq!(all[6], "Keep trails out of view: on");
        let off = Options { own: false, width: Width::Wide, ..Options::default() };
        assert_eq!(labels(off)[1], "Your trail: off");
        assert_eq!(labels(off)[5], "Width: wide");
    }

    #[test]
    fn a_remote_colour_round_trips_and_rejects_nonsense() {
        let data = encode_remote("Steve", Color::Gold);
        assert_eq!(decode_remote(&data), Some(("Steve".to_owned(), Color::Gold)));
        assert_eq!(decode_remote(b"Steve|purple"), None);
        assert_eq!(decode_remote(b"no separator"), None);
        assert_eq!(color_from_code_bytes(b"rose"), Some(Color::Rose));
        assert_eq!(color_from_code_bytes(b"\xff"), None);
    }

    #[test]
    fn the_server_tells_each_player_the_others_colours_once_they_change() {
        let (a, b) = (1_u128, 2_u128);
        assert!(server_joined(a, "Alex").is_empty());
        let (to, msg) = server_colour(a, b"gold").expect("first colour is a change");
        assert_eq!(to, Vec::<u128>::new());
        assert_eq!(msg, encode_remote("Alex", Color::Gold));
        assert!(server_colour(a, b"gold").is_none());
        let for_blake = server_joined(b, "Blake");
        assert_eq!(for_blake, vec![encode_remote("Alex", Color::Gold)]);
        let (to, msg) = server_colour(b, b"white").expect("change");
        assert_eq!(to, vec![a]);
        assert_eq!(msg, encode_remote("Blake", Color::White));
        server_left(a);
        server_left(b);
    }
}
