//! Level 1 chambers, tiles, and shared body physics.

use std::sync::OnceLock;

pub const TILE: i32 = 16;
pub const COLS: i32 = 40;
pub const ROWS: i32 = 21;
pub const VIEW_W: i32 = 640;
pub const VIEW_H: i32 = 336;
pub const HUD: i32 = 24;
pub const SCREEN_W: i32 = 640;
pub const SCREEN_H: i32 = 360;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    Solid,
    OneWay,
    Ladder,
    Vine,
    Electric,
}

impl Tile {
    fn from_char(c: char) -> Self {
        match c {
            '.' => Tile::Empty,
            '#' => Tile::Solid,
            '=' => Tile::OneWay,
            'H' => Tile::Ladder,
            'V' => Tile::Vine,
            'E' => Tile::Electric,
            _ => panic!("bad tile {c}"),
        }
    }

    pub fn is_climb(self) -> bool {
        matches!(self, Tile::Ladder | Tile::Vine)
    }

    pub fn is_full(self) -> bool {
        matches!(self, Tile::Solid | Tile::Electric)
    }
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn overlaps(self, o: Self) -> bool {
        self.x < o.x + o.w && self.x + self.w > o.x && self.y < o.y + o.h && self.y + self.h > o.y
    }
}

pub struct Room {
    pub name: &'static str,
    pub tiles: [[Tile; COLS as usize]; ROWS as usize],
    pub lanterns: [(f32, f32); 6],
    pub spawn_left: (f32, f32),
    pub spawn_right: (f32, f32),
    pub ninja: (f32, f32),
    pub yamo: (f32, f32),
    pub door_left: bool,
    pub door_right: bool,
    pub hatch: bool,
    pub statue: Option<(f32, f32)>,
    pub torches: &'static [(f32, f32)],
}

impl Room {
    pub fn tile(&self, c: i32, r: i32, hatch_open: bool) -> Tile {
        if r < 0 || r >= ROWS {
            return if r >= ROWS { Tile::Solid } else { Tile::Empty };
        }
        if c < 0 || c >= COLS {
            return Tile::Solid;
        }
        if self.hatch && hatch_open && r >= 19 && (16..23).contains(&c) {
            return Tile::Empty;
        }
        self.tiles[r as usize][c as usize]
    }
}

pub struct Body {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub vx: f32,
    pub vy: f32,
}

impl Body {
    pub fn aabb(&self) -> Rect {
        Rect {
            x: self.x - self.w * 0.5,
            y: self.y - self.h,
            w: self.w,
            h: self.h,
        }
    }
}

fn parse_map(src: &str) -> [[Tile; COLS as usize]; ROWS as usize] {
    let lines: Vec<&str> = src.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), ROWS as usize, "expected {ROWS} rows");
    let mut tiles = [[Tile::Empty; COLS as usize]; ROWS as usize];
    for (r, line) in lines.iter().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        assert_eq!(chars.len(), COLS as usize, "row {r} is {} chars", chars.len());
        for (c, ch) in chars.into_iter().enumerate() {
            tiles[r][c] = Tile::from_char(ch);
        }
    }
    tiles
}

pub fn rooms() -> &'static [Room; 3] {
    static ROOMS: OnceLock<[Room; 3]> = OnceLock::new();
    ROOMS.get_or_init(|| {
        [
            Room {
                name: "GATE COURT",
                tiles: parse_map(MAP0),
                lanterns: [(96.0, 188.0), (448.0, 188.0), (128.0, 124.0), (416.0, 124.0), (512.0, 22.0), (272.0, 150.0)],
                spawn_left: (110.0, 304.0),
                spawn_right: (580.0, 304.0),
                ninja: (416.0, 176.0),
                yamo: (530.0, 304.0),
                door_left: false,
                door_right: true,
                hatch: false,
                statue: Some((28.0, 272.0)),
                torches: &[(72.0, 112.0), (472.0, 112.0), (96.0, 240.0), (448.0, 240.0)],
            },
            Room {
                name: "MIDDLE HALL",
                tiles: parse_map(MAP1),
                lanterns: [(160.0, 36.0), (480.0, 36.0), (200.0, 112.0), (448.0, 112.0), (200.0, 176.0), (448.0, 176.0)],
                spawn_left: (48.0, 304.0),
                spawn_right: (590.0, 304.0),
                ninja: (160.0, 80.0),
                yamo: (360.0, 304.0),
                door_left: true,
                door_right: true,
                hatch: true,
                statue: None,
                torches: &[(120.0, 80.0), (520.0, 80.0), (200.0, 224.0)],
            },
            Room {
                name: "EAST GALLERY",
                tiles: parse_map(MAP2),
                lanterns: [(100.0, 248.0), (560.0, 248.0), (320.0, 240.0), (152.0, 80.0), (488.0, 80.0), (320.0, 22.0)],
                spawn_left: (48.0, 304.0),
                spawn_right: (590.0, 304.0),
                ninja: (152.0, 128.0),
                yamo: (560.0, 304.0),
                door_left: true,
                door_right: false,
                hatch: false,
                statue: None,
                torches: &[(90.0, 128.0), (540.0, 128.0)],
            },
        ]
    })
}

pub fn room(i: usize) -> &'static Room {
    &rooms()[i]
}

/// Closed-door blockers. A passage opens once the room that guards it is clear.
pub fn door_rects(room_index: usize, lanterns: &[[bool; 6]; 3]) -> Vec<Rect> {
    let room0 = lanterns[0].iter().all(|t| *t);
    let room1 = lanterns[1].iter().all(|t| *t);
    let mut v = Vec::new();
    let left = Rect { x: 0.0, y: 216.0, w: 30.0, h: 88.0 };
    let right = Rect { x: 610.0, y: 216.0, w: 30.0, h: 88.0 };
    match room_index {
        0 => {
            if !room0 {
                v.push(right);
            }
        }
        1 => {
            if !room0 {
                v.push(left);
            }
            if !room1 {
                v.push(right);
            }
        }
        _ => {
            if !room1 {
                v.push(left);
            }
        }
    }
    v
}

pub fn can_exit_right(room_index: usize, lanterns: &[[bool; 6]; 3]) -> bool {
    match room_index {
        0 => lanterns[0].iter().all(|t| *t),
        1 => lanterns[1].iter().all(|t| *t),
        _ => false,
    }
}

pub fn can_exit_left(room_index: usize, lanterns: &[[bool; 6]; 3]) -> bool {
    match room_index {
        1 => lanterns[0].iter().all(|t| *t),
        2 => lanterns[1].iter().all(|t| *t),
        _ => false,
    }
}

pub fn hatch_open(lanterns: &[[bool; 6]; 3]) -> bool {
    lanterns.iter().all(|r| r.iter().all(|t| *t))
}

pub fn elec_on(time: f32) -> bool {
    let t = time.rem_euclid(2.45);
    t >= 1.75
}

pub fn elec_warn(time: f32) -> bool {
    let t = time.rem_euclid(2.45);
    t >= 1.35 && t < 1.75
}

pub fn climb_snap(room: &Room, x: f32, feet: f32) -> Option<f32> {
    let r = ((feet - 22.0) / TILE as f32).floor() as i32;
    let c = (x / TILE as f32).floor() as i32;
    let mut best: Option<(i32, i32)> = None;
    for dr in -1..=1 {
        for dc in -1..=1 {
            let cc = c + dc;
            let rr = r + dr;
            if room.tile(cc, rr, false).is_climb() {
                let dist = (x - (cc as f32 + 0.5) * TILE as f32).abs() as i32;
                if best.map(|(d, _)| dist < d).unwrap_or(true) {
                    best = Some((dist, cc));
                }
            }
        }
    }
    best.map(|(_, cc)| (cc as f32 + 0.5) * TILE as f32)
}

pub fn on_electric(room: &Room, x: f32, feet: f32) -> bool {
    let c = (x / TILE as f32).floor() as i32;
    let r = ((feet + 2.0) / TILE as f32).floor() as i32;
    room.tile(c, r, false) == Tile::Electric
}

/// Move a body. Returns true when the feet are planted on a surface.
pub fn step_body(body: &mut Body, room: &Room, gravity: bool, doors: &[Rect], hatch_open: bool, dt: f32) -> bool {
    if gravity {
        body.vy += 1480.0 * dt;
        if body.vy > 780.0 {
            body.vy = 780.0;
        }
    }
    body.x += body.vx * dt;
    resolve_x(body, room, doors, hatch_open);

    let prev_feet = body.y;
    body.y += body.vy * dt;
    let grounded = resolve_y(body, room, doors, hatch_open, prev_feet);
    if grounded {
        body.vy = 0.0;
    }
    grounded
}

fn resolve_x(body: &mut Body, room: &Room, doors: &[Rect], hatch_open: bool) {
    let mut guard = 0;
    while guard < 4 {
        guard += 1;
        let rect = body.aabb();
        let mut fix: Option<f32> = None;
        each_full(room, doors, hatch_open, rect, |tr| {
            if body.vx > 0.0 {
                let nx = tr.x - body.w * 0.5;
                fix = Some(fix.map(|v: f32| v.min(nx)).unwrap_or(nx));
            } else if body.vx < 0.0 {
                let nx = tr.x + tr.w + body.w * 0.5;
                fix = Some(fix.map(|v: f32| v.max(nx)).unwrap_or(nx));
            } else {
                let pen_l = rect.x + rect.w - tr.x;
                let pen_r = tr.x + tr.w - rect.x;
                if pen_l < pen_r {
                    let nx = tr.x - body.w * 0.5;
                    fix = Some(fix.map(|v: f32| v.min(nx)).unwrap_or(nx));
                } else {
                    let nx = tr.x + tr.w + body.w * 0.5;
                    fix = Some(fix.map(|v: f32| v.max(nx)).unwrap_or(nx));
                }
            }
        });
        if let Some(nx) = fix {
            body.x = nx;
            body.vx = 0.0;
        } else {
            break;
        }
    }
}

fn resolve_y(body: &mut Body, room: &Room, doors: &[Rect], hatch_open: bool, prev_feet: f32) -> bool {
    let mut land: Option<f32> = None;
    if body.vy >= 0.0 {
        let rect = body.aabb();
        let c0 = (rect.x / TILE as f32).floor() as i32 - 1;
        let c1 = ((rect.x + rect.w) / TILE as f32).ceil() as i32 + 1;
        let r0 = (rect.y / TILE as f32).floor() as i32 - 1;
        let r1 = ((rect.y + rect.h) / TILE as f32).ceil() as i32 + 1;
        for r in r0..=r1 {
            for c in c0..=c1 {
                let tile = room.tile(c, r, hatch_open);
                let stand = tile.is_full() || tile == Tile::OneWay;
                if !stand {
                    continue;
                }
                let top = r as f32 * TILE as f32;
                let left = c as f32 * TILE as f32;
                let right = left + TILE as f32;
                let x_overlap = body.x + body.w * 0.5 > left + 1.0 && body.x - body.w * 0.5 < right - 1.0;
                if x_overlap && prev_feet <= top + 2.0 && body.y >= top {
                    land = Some(match land {
                        Some(y) if y < top => y,
                        _ => top,
                    });
                }
            }
        }
        for tr in doors {
            let top = tr.y;
            let x_overlap = body.x + body.w * 0.5 > tr.x && body.x - body.w * 0.5 < tr.x + tr.w;
            if x_overlap && prev_feet <= top + 2.0 && body.y >= top {
                land = Some(match land {
                    Some(y) if y < top => y,
                    _ => top,
                });
            }
        }
    }
    if let Some(top) = land {
        body.y = top;
        return true;
    }

    // Head bonk on full solids only.
    if body.vy < 0.0 {
        let rect = body.aabb();
        let mut bonk: Option<f32> = None;
        each_full(room, doors, hatch_open, rect, |tr| {
            let prev_head = prev_feet - body.h;
            if prev_head >= tr.y + tr.h - 2.0 {
                let ny = tr.y + tr.h + body.h;
                bonk = Some(bonk.map(|v: f32| v.max(ny)).unwrap_or(ny));
            }
        });
        if let Some(ny) = bonk {
            body.y = ny;
            body.vy = 0.0;
        }
    }
    false
}

fn each_full(room: &Room, doors: &[Rect], hatch_open: bool, rect: Rect, mut f: impl FnMut(Rect)) {
    let c0 = (rect.x / TILE as f32).floor() as i32 - 1;
    let c1 = ((rect.x + rect.w) / TILE as f32).ceil() as i32 + 1;
    let r0 = (rect.y / TILE as f32).floor() as i32 - 1;
    let r1 = ((rect.y + rect.h) / TILE as f32).ceil() as i32 + 1;
    for r in r0..=r1 {
        for c in c0..=c1 {
            if !room.tile(c, r, hatch_open).is_full() {
                continue;
            }
            let tr = Rect {
                x: c as f32 * TILE as f32,
                y: r as f32 * TILE as f32,
                w: TILE as f32,
                h: TILE as f32,
            };
            if rect.overlaps(tr) {
                f(tr);
            }
        }
    }
    for tr in doors {
        if rect.overlaps(*tr) {
            f(*tr);
        }
    }
}

#[cfg(test)]
struct Plat {
    y: i32,
    x0: i32,
    x1: i32,
}
#[cfg(test)]
struct Climb {
    x: i32,
    y0: i32,
    y1: i32,
}

#[cfg(test)]
fn standable(room: &Room, c: i32, r: i32) -> bool {
    let t = room.tile(c, r, false);
    if !(t.is_full() || t == Tile::OneWay) {
        return false;
    }
    if r == 0 {
        return true;
    }
    !room.tile(c, r - 1, false).is_full()
}

#[cfg(test)]
fn platforms(room: &Room) -> Vec<Plat> {
    let mut v = Vec::new();
    for r in 0..ROWS {
        let mut c = 0;
        while c < COLS {
            if standable(room, c, r) {
                let c0 = c;
                while c < COLS && standable(room, c, r) {
                    c += 1;
                }
                v.push(Plat { y: r * TILE, x0: c0 * TILE, x1: c * TILE });
            } else {
                c += 1;
            }
        }
    }
    v
}

#[cfg(test)]
fn climbs(room: &Room) -> Vec<Climb> {
    let mut v = Vec::new();
    for c in 0..COLS {
        let mut r = 0;
        while r < ROWS {
            if room.tile(c, r, false).is_climb() {
                let r0 = r;
                while r < ROWS && room.tile(c, r, false).is_climb() {
                    r += 1;
                }
                v.push(Climb { x: c * TILE + TILE / 2, y0: r0 * TILE, y1: r * TILE });
            } else {
                r += 1;
            }
        }
    }
    v
}

#[cfg(test)]
fn x_gap(a0: i32, a1: i32, b0: i32, b1: i32) -> i32 {
    if a1 < b0 {
        b0 - a1
    } else if b1 < a0 {
        a0 - b1
    } else {
        0
    }
}

/// Every lantern can be reached from the left-hand spawn by jumps and climbs.
#[cfg(test)]
fn unreachable_lanterns(room_index: usize) -> Vec<usize> {
    let room = room(room_index);
    let plats = platforms(room);
    let cols = climbs(room);
    let spawn = room.spawn_left;
    let start = plats.iter().position(|p| {
        spawn.0 >= p.x0 as f32 && spawn.0 < p.x1 as f32 && (spawn.1 - p.y as f32).abs() < 1.0
    });
    let Some(start) = start else {
        return (0..6).collect();
    };
    let n_p = plats.len();
    let n_c = cols.len();
    let mut reach_p = vec![false; n_p];
    let mut reach_c = vec![false; n_c];
    reach_p[start] = true;
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..n_p {
            if !reach_p[i] {
                continue;
            }
            for j in 0..n_p {
                if reach_p[j] || i == j {
                    continue;
                }
                let g = x_gap(plats[i].x0, plats[i].x1, plats[j].x0, plats[j].x1);
                let dy = plats[j].y - plats[i].y;
                let ok = (dy > 0 && g <= 80) || (dy < 0 && -dy <= 92 && g <= 108) || (dy.abs() <= 6 && g > 0 && g <= 108);
                if ok {
                    reach_p[j] = true;
                    changed = true;
                }
            }
            for j in 0..n_c {
                if reach_c[j] {
                    continue;
                }
                let p = &plats[i];
                let c = &cols[j];
                if c.x >= p.x0 - 24 && c.x <= p.x1 + 24 && p.y + 10 >= c.y0 && p.y - 96 <= c.y1 {
                    reach_c[j] = true;
                    changed = true;
                }
            }
        }
        for i in 0..n_c {
            if !reach_c[i] {
                continue;
            }
            for j in 0..n_c {
                if reach_c[j] || i == j {
                    continue;
                }
                let a = &cols[i];
                let b = &cols[j];
                let y_over = a.y0 < b.y1 && b.y0 < a.y1;
                if (a.x - b.x).abs() <= 20 && y_over {
                    reach_c[j] = true;
                    changed = true;
                }
            }
            for j in 0..n_p {
                if reach_p[j] {
                    continue;
                }
                let p = &plats[j];
                let c = &cols[i];
                if c.x >= p.x0 - 24 && c.x <= p.x1 + 24 && p.y + 10 >= c.y0 && p.y - 96 <= c.y1 {
                    reach_p[j] = true;
                    changed = true;
                }
            }
        }
    }
    let mut bad = Vec::new();
    for (i, &(lx, ly)) in room.lanterns.iter().enumerate() {
        let lx_i = lx as i32;
        let ly_i = ly as i32;
        let ok_p = plats.iter().enumerate().any(|(pi, p)| {
            if !reach_p[pi] {
                return false;
            }
            let dx = if lx_i < p.x0 {
                p.x0 - lx_i
            } else if lx_i > p.x1 {
                lx_i - p.x1
            } else {
                0
            };
            let dy = p.y - ly_i;
            dx <= 44 && (4..=100).contains(&dy)
        });
        let ok_c = cols.iter().enumerate().any(|(ci, c)| {
            reach_c[ci] && (lx_i - c.x).abs() <= 30 && ly_i >= c.y0 - 24 && ly_i <= c.y1 + 8
        });
        if !ok_p && !ok_c {
            bad.push(i);
        }
    }
    bad
}

const MAP0: &str = "\
........................................
........................................
........................................
........................................
====.....=======VV=======.....====......
................VV......................
................VV......................
..=====.........VV.........=====........
................VV......................
................VV......................
................VV......................
.....======.....VV.....======...........
................VV......................
................VV......................
................VV......................
...======.......VV.......======.........
................VV......................
###.............VV......................
###.............VV......................
########################################
########################################
";

const MAP1: &str = "\
........................................
........................................
........................................
........................................
........................................
....H==========....VV....==========H....
....H..............VV..............H....
....H..............VV..............H....
....H..............VV..............H....
....H..............VV..............H....
....H.=============VV==============H....
....H..............VV..............H....
....H..............VV..............H....
....H..............VV..............H....
....H.=============VV==============H....
....H..............................H....
....H..............................H....
....H..............................H....
....H..............................H....
########################################
########################################
";

const MAP2: &str = "\
........................................
........................................
........................................
........................................
........========================........
..H..................................H..
..H..................................H..
..H..................................H..
..H=============........=============H..
..H..................................H..
..H..................................H..
..H..................................H..
..H..................................H..
..H..................................H..
..H..................................H..
..H..................................H..
..H..................................H..
..H..................................H..
..H..................................H..
#############EEEEEEEEEEEEE##############
#############EEEEEEEEEEEEE##############
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_lantern_is_reachable() {
        for i in 0..3 {
            let bad = unreachable_lanterns(i);
            assert!(bad.is_empty(), "room {i} lanterns {bad:?} unreachable");
        }
    }

    #[test]
    fn falling_lands_on_the_court_floor() {
        let room = room(0);
        let mut b = Body { x: 200.0, y: 180.0, w: 16.0, h: 42.0, vx: 0.0, vy: 0.0 };
        let mut grounded = false;
        for _ in 0..240 {
            grounded = step_body(&mut b, room, true, &[], false, 1.0 / 60.0);
            if grounded && (b.y - 304.0).abs() < 0.1 {
                break;
            }
        }
        assert!(grounded, "never landed, y={}", b.y);
        assert!((b.y - 304.0).abs() < 0.1, "y {}", b.y);
    }

    #[test]
    fn jump_clears_a_ledge_and_one_way_platforms_can_be_passed() {
        let room = room(0);
        let mut b = Body { x: 96.0, y: 270.0, w: 16.0, h: 42.0, vx: 0.0, vy: -520.0 };
        let mut min_y = b.y;
        let mut landed = false;
        for _ in 0..300 {
            let g = step_body(&mut b, room, true, &[], false, 1.0 / 60.0);
            min_y = min_y.min(b.y);
            if g && (b.y - 240.0).abs() < 0.5 {
                landed = true;
                break;
            }
        }
        assert!(min_y < 210.0, "did not pass above the ledge, min {min_y}");
        assert!(landed, "did not land on the ledge, y={}", b.y);
    }
}
