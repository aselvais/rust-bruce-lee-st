//! Four fortress levels. Each level is three chambers, lanterns, the ninja, and Green Yamo.

use crate::gfx::{self, Assets, Frame};
use crate::level::{self, Body, Room};

const GRAVITY_DT_SCALE: f32 = 1.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Title,
    Play,
    Pause,
    Clear,
    Over,
}

/// One-shot effects. Music follows `Mode` and is not queued here.
#[derive(Clone, Copy)]
pub enum Sfx {
    Jump,
    Punch,
    Kick,
    Lantern,
    Hit,
    Hurt,
    Ko,
    Zap,
    Open,
    Throw,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Act {
    Idle,
    Run,
    Jump,
    Fall,
    Duck,
    Climb,
    Punch,
    Kick,
    Hurt,
    Dead,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EKind {
    Ninja,
    Yamo,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EMode {
    Dormant,
    Appear,
    Chase,
    Attack,
    Throw,
    Hurt,
    Dead,
}

pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub jump_edge: bool,
    pub jump_held: bool,
    pub up_edge: bool,
    pub attack_edge: bool,
    pub start_edge: bool,
    pub pause_edge: bool,
}

struct Fighter {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    w: f32,
    h: f32,
    facing: i32,
    act: Act,
    act_t: f32,
    grounded: bool,
    coyote: f32,
    climbing: bool,
    invuln: f32,
    anim: f32,
    attack_hit: bool,
    energy: i32,
    shock: f32,
}

struct Enemy {
    kind: EKind,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    facing: i32,
    hp: i32,
    max_hp: i32,
    grounded: bool,
    climbing: bool,
    mode: EMode,
    mode_t: f32,
    anim: f32,
    cd: f32,
    hit: bool,
    shock: f32,
}

struct RoomRun {
    lanterns: [bool; 6],
    ninja: Enemy,
    yamo: Enemy,
    woke: bool,
}

struct Slide {
    from: usize,
    to: usize,
    dir: i32,
    t: f32,
}

struct Star {
    x: f32,
    y: f32,
    vx: f32,
    life: f32,
    spin: f32,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    color: u32,
}

struct Floater {
    x: f32,
    y: f32,
    text: String,
    life: f32,
    color: u32,
}

pub struct Game {
    pub mode: Mode,
    pub level: usize,
    pub room: usize,
    pub time: f32,
    pub score: u32,
    pub lives: i32,
    pub show_hit: bool,
    player: Fighter,
    runs: [RoomRun; 3],
    slide: Option<Slide>,
    stars: Vec<Star>,
    particles: Vec<Particle>,
    floaters: Vec<Floater>,
    banner: String,
    banner_t: f32,
    visited: [bool; 3],
    hint: f32,
    shake: f32,
    dead_t: f32,
    sounds: Vec<Sfx>,
}

impl Game {
    pub fn new() -> Self {
        let mut g = Self {
            mode: Mode::Title,
            level: 0,
            room: 0,
            time: 0.0,
            score: 0,
            lives: 5,
            show_hit: false,
            player: Fighter::hero(level::room(0, 0).spawn_left),
            runs: RoomRun::set(0),
            slide: None,
            stars: Vec::new(),
            particles: Vec::new(),
            floaters: Vec::new(),
            banner: String::new(),
            banner_t: 0.0,
            visited: [true, false, false],
            hint: 7.0,
            shake: 0.0,
            dead_t: 0.0,
            sounds: Vec::new(),
        };
        g.runs[0].woke = true;
        g
    }

    pub fn take_sounds(&mut self) -> Vec<Sfx> {
        std::mem::take(&mut self.sounds)
    }

    fn cue(&mut self, s: Sfx) {
        if self.sounds.len() < 12 {
            self.sounds.push(s);
        }
    }

    pub fn reset_play(&mut self) {
        *self = Self::new();
        self.mode = Mode::Play;
    }

    pub fn update(&mut self, input: &Input, dt: f32) {
        self.time += dt;
        self.shake = (self.shake - dt * 1.6).max(0.0);
        self.banner_t = (self.banner_t - dt).max(0.0);
        self.hint = (self.hint - dt).max(0.0);
        tick_fx(&mut self.particles, &mut self.floaters, dt);

        match self.mode {
            Mode::Title => {
                if input.start_edge {
                    self.reset_play();
                    self.cue(Sfx::Open);
                }
                return;
            }
            Mode::Pause => {
                if input.pause_edge || input.start_edge {
                    self.mode = Mode::Play;
                }
                return;
            }
            Mode::Clear | Mode::Over => {
                if input.start_edge {
                    self.reset_play();
                    self.cue(Sfx::Open);
                }
                return;
            }
            Mode::Play => {}
        }
        if input.pause_edge {
            self.mode = Mode::Pause;
            return;
        }
        if let Some(slide) = self.slide.as_mut() {
            slide.t += dt / 0.38;
            if slide.t >= 1.0 {
                let to = slide.to;
                let dir = slide.dir;
                self.slide = None;
                self.arrive(to, dir);
            }
            return;
        }

        let hatch = level::hatch_open(&self.lantern_flags());
        let doors = level::door_rects(self.room, &self.lantern_flags());
        self.update_player(input, dt, &doors, hatch);
        self.update_enemies(dt, &doors, hatch);
        self.collect_lanterns();
        self.combat();
        self.update_stars(dt);
        self.hazards(dt);
        self.try_exit();
        if self.player.act == Act::Dead {
            self.dead_t += dt;
            if self.dead_t > 0.9 {
                if self.lives <= 0 {
                    self.mode = Mode::Over;
                } else {
                    self.respawn();
                }
            }
        }
    }

    fn lantern_flags(&self) -> [[bool; 6]; 3] {
        [self.runs[0].lanterns, self.runs[1].lanterns, self.runs[2].lanterns]
    }

    fn arrive(&mut self, to: usize, dir: i32) {
        self.room = to;
        let dest = level::room(self.level, to);
        let (sx, sy) = if dir > 0 { dest.spawn_left } else { dest.spawn_right };
        self.player.y = sy;
        self.player.vy = 0.0;
        self.player.vx = 0.0;
        self.player.climbing = false;
        self.player.act = Act::Idle;
        self.player.x = sx;
        self.stars.clear();
        if !self.visited[to] {
            self.visited[to] = true;
            self.score += 2000;
            self.banner("NEW CHAMBER +2000");
            self.cue(Sfx::Open);
            self.float_at(self.player.x, self.player.y - 60.0, "+2000", 0xE6B43A);
        }
        let run = &mut self.runs[to];
        if !run.woke {
            run.woke = true;
            run.ninja.mode = EMode::Dormant;
            run.ninja.mode_t = 1.5;
            run.yamo.mode = EMode::Dormant;
            run.yamo.mode_t = 3.4;
        }
    }

    fn update_player(&mut self, input: &Input, dt: f32, doors: &[level::Rect], hatch: bool) {
        let mut cues: Vec<Sfx> = Vec::new();
        let p = &mut self.player;
        p.anim += dt;
        p.invuln = (p.invuln - dt).max(0.0);
        p.shock = (p.shock - dt).max(0.0);
        p.coyote = (p.coyote - dt).max(0.0);
        if p.act == Act::Dead {
            let room = level::room(self.level, self.room);
            let mut body = p.body();
            p.grounded = level::step_body(&mut body, room, true, doors, hatch, dt);
            p.copy_body(&body);
            let _ = p;
            self.sounds.append(&mut cues);
            return;
        }
        if p.act == Act::Hurt {
            p.act_t += dt;
            if p.act_t > 0.28 {
                p.act = Act::Fall;
            }
        }

        let attacking = matches!(p.act, Act::Punch | Act::Kick);
        if attacking {
            p.act_t += dt;
            let dur = if p.act == Act::Kick { 0.34 } else { 0.30 };
            if p.act == Act::Kick {
                p.vx = p.facing as f32 * 220.0;
            }
            if p.act_t > dur {
                p.act = if p.grounded { Act::Idle } else { Act::Fall };
            }
        }

        let room = level::room(self.level, self.room);
        if p.climbing {
            p.vx = 0.0;
            p.vy = 0.0;
            if input.up {
                p.y -= 92.0 * dt;
            }
            if input.down {
                p.y += 92.0 * dt;
            }
            p.act = Act::Climb;
            if input.jump_edge {
                p.climbing = false;
                p.vy = -280.0;
                p.vx = p.facing as f32 * 90.0;
                p.act = Act::Jump;
            } else if input.left || input.right {
                let dir = if input.left { -1.0 } else { 1.0 };
                p.facing = if input.left { -1 } else { 1 };
                let nx = p.x + dir * 14.0;
                if let Some(top) = surface_near(room, nx, p.y) {
                    p.climbing = false;
                    p.x = nx;
                    p.y = top;
                    p.grounded = true;
                    p.act = Act::Idle;
                }
            }
            if level::climb_snap(room, p.x, p.y).is_none() {
                p.climbing = false;
            }
            if let Some(top) = surface_near(room, p.x, p.y) {
                if p.y >= top - 1.0 && input.down {
                    p.climbing = false;
                    p.y = top;
                    p.grounded = true;
                }
            }
            // Don't fall through the floor while holding down on a vine.
            if p.y > 304.0 {
                p.y = 304.0;
                p.climbing = false;
                p.grounded = true;
            }
            let _ = p;
            self.sounds.append(&mut cues);
            return;
        }

        let want_duck = input.down && p.grounded && !attacking;
        p.h = if want_duck { 22.0 } else { 42.0 };
        if want_duck {
            p.act = Act::Duck;
            p.vx = 0.0;
        }

        if !attacking && !want_duck && p.act != Act::Hurt {
            let mut ax = 0.0;
            if input.left {
                ax -= 1.0;
                p.facing = -1;
            }
            if input.right {
                ax += 1.0;
                p.facing = 1;
            }
            let accel = if p.grounded { 2400.0 } else { 980.0 };
            if ax != 0.0 {
                p.vx += ax * accel * dt;
                p.vx = p.vx.clamp(-175.0, 175.0);
            } else if p.grounded {
                let f = (16.0 * dt).min(1.0);
                p.vx *= 1.0 - f;
                if p.vx.abs() < 10.0 {
                    p.vx = 0.0;
                }
            }
        }

        if input.attack_edge && !attacking && p.act != Act::Hurt {
            p.attack_hit = false;
            p.act_t = 0.0;
            if p.grounded && p.vx.abs() < 40.0 {
                p.act = Act::Punch;
                p.vx = 0.0;
                cues.push(Sfx::Punch);
            } else {
                p.act = Act::Kick;
                cues.push(Sfx::Kick);
                p.vx = p.facing as f32 * 230.0;
                if p.grounded {
                    p.vy = -80.0;
                    p.grounded = false;
                }
            }
            p.climbing = false;
            p.h = 42.0;
        }

        if input.up && !attacking && p.act != Act::Hurt {
            if let Some(sx) = level::climb_snap(room, p.x, p.y) {
                p.climbing = true;
                p.x = sx;
                p.vx = 0.0;
                p.vy = 0.0;
                p.act = Act::Climb;
                p.h = 42.0;
                let _ = p;
                self.sounds.append(&mut cues);
                return;
            }
        }

        if (input.jump_edge || input.up_edge) && (p.grounded || p.coyote > 0.0) && !matches!(p.act, Act::Punch | Act::Kick) {
            p.vy = -505.0;
            p.grounded = false;
            p.coyote = 0.0;
            p.act = Act::Jump;
            p.h = 42.0;
            cues.push(Sfx::Jump);
        }
        if !input.jump_held && p.vy < -160.0 {
            p.vy = -160.0;
        }

        let mut body = p.body();
        let was_ground = p.grounded;
        p.grounded = level::step_body(&mut body, room, true, doors, hatch, dt);
        p.copy_body(&body);
        if was_ground && !p.grounded && p.vy >= 0.0 {
            p.coyote = 0.09;
        }
        if p.grounded && p.vy.abs() < 1.0 && !attacking && p.act != Act::Hurt && p.act != Act::Duck {
            p.act = if p.vx.abs() > 20.0 { Act::Run } else { Act::Idle };
        } else if !p.grounded && !attacking && !p.climbing && p.act != Act::Hurt && p.act != Act::Dead {
            p.act = if p.vy < 0.0 { Act::Jump } else { Act::Fall };
        }
        let _ = GRAVITY_DT_SCALE;
        let _ = p;
        self.sounds.append(&mut cues);
    }

    fn update_enemies(&mut self, dt: f32, doors: &[level::Rect], hatch: bool) {
        let room_i = self.room;
        let px = self.player.x;
        let py = self.player.y;
        let player_alive = self.player.act != Act::Dead;
        let room = level::room(self.level, room_i);
        let mut spawned: Vec<Star> = Vec::new();
        for which in 0..2 {
            let e = if which == 0 { &mut self.runs[room_i].ninja } else { &mut self.runs[room_i].yamo };
            e.anim += dt;
            e.cd = (e.cd - dt).max(0.0);
            e.shock = (e.shock - dt).max(0.0);
            match e.mode {
                EMode::Dormant => {
                    e.mode_t -= dt;
                    if e.mode_t <= 0.0 {
                        e.mode = EMode::Appear;
                        e.mode_t = 0.45;
                        let sp = if which == 0 { room.ninja } else { room.yamo };
                        e.x = sp.0;
                        e.y = sp.1;
                        e.vx = 0.0;
                        e.vy = 0.0;
                        e.hp = e.max_hp;
                    }
                }
                EMode::Appear => {
                    e.mode_t -= dt;
                    if e.mode_t <= 0.0 {
                        e.mode = EMode::Chase;
                    }
                }
                EMode::Hurt => {
                    e.mode_t -= dt;
                    physics_enemy(e, room, doors, hatch, dt);
                    if e.mode_t <= 0.0 {
                        e.mode = EMode::Chase;
                    }
                }
                EMode::Dead => {
                    e.mode_t -= dt;
                    physics_enemy(e, room, doors, hatch, dt);
                    if e.mode_t <= 0.0 {
                        let sp = if which == 0 { room.ninja } else { room.yamo };
                        e.x = sp.0;
                        e.y = sp.1;
                        e.hp = e.max_hp;
                        e.mode = EMode::Dormant;
                        e.mode_t = 0.6;
                        e.climbing = false;
                        e.vx = 0.0;
                        e.vy = 0.0;
                    }
                }
                EMode::Attack => {
                    e.mode_t += dt;
                    let dur = if e.kind == EKind::Ninja { 0.32 } else { 0.40 };
                    physics_enemy(e, room, doors, hatch, dt);
                    if e.mode_t > dur {
                        e.mode = EMode::Chase;
                        e.cd = if e.kind == EKind::Ninja { 0.55 } else { 0.75 };
                    }
                }
                EMode::Throw => {
                    e.mode_t += dt;
                    e.vx = 0.0;
                    physics_enemy(e, room, doors, hatch, dt);
                    if !e.hit && e.mode_t >= 0.14 {
                        e.hit = true;
                        let dir = e.facing as f32;
                        spawned.push(Star {
                            x: e.x + dir * 18.0,
                            y: e.y - 30.0,
                            vx: dir * 230.0,
                            life: 1.5,
                            spin: 0.0,
                        });
                    }
                    if e.mode_t > 0.42 {
                        e.mode = EMode::Chase;
                        e.cd = 1.15;
                    }
                }
                EMode::Chase if player_alive => {
                    let dir = if px < e.x { -1.0 } else { 1.0 };
                    e.facing = if dir < 0.0 { -1 } else { 1 };
                    let dist = (px - e.x).abs();
                    let same_floor = (py - e.y).abs() < 40.0;
                    if e.kind == EKind::Ninja
                        && e.grounded
                        && !e.climbing
                        && e.cd <= 0.0
                        && same_floor
                        && dist > 56.0
                        && dist < 250.0
                    {
                        e.mode = EMode::Throw;
                        e.mode_t = 0.0;
                        e.hit = false;
                        e.vx = 0.0;
                        continue;
                    }
                    let range = if e.kind == EKind::Ninja { 40.0 } else { 46.0 };
                    if e.grounded && dist < range && same_floor && e.cd <= 0.0 {
                        e.mode = EMode::Attack;
                        e.mode_t = 0.0;
                        e.hit = false;
                        e.vx = 0.0;
                        continue;
                    }
                    if let Some(sx) = level::climb_snap(room, e.x, e.y) {
                        if (py - e.y).abs() > 56.0 && (px - e.x).abs() < 110.0 {
                            e.climbing = true;
                            e.x = sx;
                        }
                    }
                    if e.climbing {
                        let v = if py + 10.0 < e.y { -78.0 } else { 78.0 };
                        e.y += v * dt;
                        e.vx = 0.0;
                        if (py - e.y).abs() < 20.0 || level::climb_snap(room, e.x, e.y).is_none() || e.y >= 304.0 {
                            e.climbing = false;
                            if e.y > 304.0 {
                                e.y = 304.0;
                            }
                        }
                        continue;
                    }
                    let speed = if e.kind == EKind::Ninja { 108.0 } else { 78.0 };
                    e.vx = dir * speed;
                    let probe_x = e.x + dir * 16.0;
                    let blocked = solid_point(room, probe_x, e.y - 10.0, hatch) && solid_point(room, probe_x, e.y - 28.0, hatch);
                    if e.grounded && blocked && py < e.y - 24.0 && e.cd <= 0.0 {
                        e.vy = -470.0;
                        e.cd = 0.4;
                    }
                    physics_enemy(e, room, doors, hatch, dt);
                }
                EMode::Chase => {
                    e.vx = 0.0;
                    physics_enemy(e, room, doors, hatch, dt);
                }
            }
        }
        // Separate so they don't stack into one sprite.
        let run = &mut self.runs[room_i];
        if run.ninja.alive_body() && run.yamo.alive_body() && (run.ninja.x - run.yamo.x).abs() < 22.0 && (run.ninja.y - run.yamo.y).abs() < 30.0 {
            let push = if run.ninja.x <= run.yamo.x { -1.0 } else { 1.0 };
            run.ninja.x += push * 30.0 * dt;
            run.yamo.x -= push * 30.0 * dt;
        }
        let thrown = spawned.len();
        self.stars.extend(spawned);
        for _ in 0..thrown {
            self.cue(Sfx::Throw);
        }
    }

    fn collect_lanterns(&mut self) {
        if self.player.act == Act::Dead {
            return;
        }
        let flags = self.lantern_flags();
        let room_i = self.room;
        let room = level::room(self.level, room_i);
        let px = self.player.x;
        let py = self.player.y - self.player.h * 0.45;
        let mut got = Vec::new();
        for (i, &(lx, ly)) in room.lanterns.iter().enumerate() {
            if self.runs[room_i].lanterns[i] {
                continue;
            }
            let dx = px - lx;
            let dy = py - ly;
            if dx * dx + dy * dy < 26.0 * 26.0 {
                got.push(i);
            }
        }
        let mut newly_room = false;
        let mut newly_all = false;
        for i in got {
            self.runs[room_i].lanterns[i] = true;
            self.score += 125;
            self.float_at(room.lanterns[i].0, room.lanterns[i].1, "+125", 0xFFD060);
            self.burst(room.lanterns[i].0, room.lanterns[i].1, 0xFFC040);
            self.cue(Sfx::Lantern);
            newly_room = self.runs[room_i].lanterns.iter().all(|t| *t);
            newly_all = level::hatch_open(&self.lantern_flags());
        }
        if newly_room {
            self.shake = 0.35;
            self.cue(Sfx::Open);
            if newly_all {
                self.banner("THE HALL FLOOR IS OPEN");
            } else if room_i == 2 {
                self.banner("RETURN TO THE HALL");
            } else {
                self.banner("PASSAGE OPEN");
            }
        }
        let _ = flags;
    }

    fn combat(&mut self) {
        if self.player.act == Act::Dead {
            return;
        }
        let room_i = self.room;
        if self.player.vy > 60.0 {
            for which in 0..2 {
                let stomp = {
                    let e = enemy_ref(&self.runs[room_i], which);
                    if !e.hurtable() {
                        false
                    } else {
                        let eh = if e.kind == EKind::Yamo { 48.0 } else { 40.0 };
                        let ew = if e.kind == EKind::Yamo { 26.0 } else { 16.0 };
                        let head = e.y - eh;
                        self.player.y > head - 2.0
                            && self.player.y < head + 14.0
                            && (self.player.x - e.x).abs() < (self.player.w + ew) * 0.5
                    }
                };
                if stomp {
                    self.hurt_enemy(which, 50, true);
                    self.player.vy = -280.0;
                    self.player.grounded = false;
                }
            }
        }
        if let Some(hit) = self.player.attack_box() {
            if !self.player.attack_hit {
                for which in 0..2 {
                    let connected = {
                        let e = enemy_ref(&self.runs[room_i], which);
                        e.hurtable() && hit.overlaps(e.aabb())
                    };
                    if connected {
                        let pts = if self.player.act == Act::Kick { 75 } else { 100 };
                        self.player.attack_hit = true;
                        self.hurt_enemy(which, pts, false);
                    }
                }
            }
        }
        let ducking = self.player.h < 30.0;
        let player_box = self.player.aabb();
        let mut hits: Vec<(usize, bool, Option<usize>)> = Vec::new();
        for which in 0..2 {
            let e = enemy_ref(&self.runs[room_i], which);
            if e.mode == EMode::Attack && !e.hit {
                if let Some(box_) = e.attack_box() {
                    let high = e.kind == EKind::Ninja;
                    let connect = box_.overlaps(player_box) && !(high && ducking) && self.player.invuln <= 0.0;
                    let other = 1 - which;
                    let o = enemy_ref(&self.runs[room_i], other);
                    let friend = o.hurtable() && box_.overlaps(o.aabb());
                    if connect || friend {
                        hits.push((which, connect, if friend { Some(other) } else { None }));
                    }
                }
            }
        }
        for (which, connect, friend) in hits {
            if which == 0 {
                self.runs[room_i].ninja.hit = true;
            } else {
                self.runs[room_i].yamo.hit = true;
            }
            if connect {
                let dir = if self.player.x < enemy_ref(&self.runs[room_i], which).x { -1 } else { 1 };
                self.damage_player(dir);
            }
            if let Some(other) = friend {
                self.hurt_enemy(other, 75, false);
            }
        }
        if self.player.invuln <= 0.0 && self.player.act != Act::Dead {
            for which in 0..2 {
                let touch = {
                    let e = enemy_ref(&self.runs[room_i], which);
                    e.mode == EMode::Chase && player_box.overlaps(e.aabb())
                };
                if touch {
                    let dir = if self.player.x < enemy_ref(&self.runs[room_i], which).x { -1 } else { 1 };
                    self.damage_player(dir);
                    break;
                }
            }
        }
    }

    fn hurt_enemy(&mut self, which: usize, points: u32, stomp: bool) {
        let room_i = self.room;
        let outcome = {
            let e = enemy_mut(&mut self.runs[room_i], which);
            if !e.hurtable() {
                return;
            }
            e.hp -= 1;
            let dead = e.hp <= 0;
            let dir = if e.x < self.player.x { -1.0 } else { 1.0 };
            if dead {
                e.mode = EMode::Dead;
                e.mode_t = 3.2;
                e.vx = dir * 40.0;
                e.vy = -80.0;
            } else {
                e.mode = EMode::Hurt;
                e.mode_t = 0.28;
                e.vx = dir * 140.0;
                e.vy = if stomp { -40.0 } else { -120.0 };
            }
            e.climbing = false;
            (e.x, e.y, e.kind, dead)
        };
        let (ex, ey, kind, dead) = outcome;
        if dead {
            let bonus = if kind == EKind::Ninja { 200 } else { 450 };
            self.score += points + bonus;
            self.float_at(ex, ey - 50.0, &format!("+{}", points + bonus), 0xFFD060);
        } else {
            self.score += points;
            self.float_at(ex, ey - 46.0, &format!("+{points}"), 0xF3E6C4);
        }
        self.burst(ex, ey - 24.0, 0xF2E2C0);
        self.shake = self.shake.max(0.12);
        self.cue(if dead { Sfx::Ko } else { Sfx::Hit });
    }

    fn damage_player(&mut self, dir_from_enemy: i32) {
        if self.player.invuln > 0.0 || self.player.act == Act::Dead {
            return;
        }
        self.player.energy -= 1;
        self.player.invuln = 1.05;
        self.player.vx = dir_from_enemy as f32 * -160.0;
        self.player.vy = -200.0;
        self.player.climbing = false;
        self.player.grounded = false;
        self.shake = self.shake.max(0.22);
        self.burst(self.player.x, self.player.y - 24.0, 0xE07050);
        self.cue(if self.player.shock > 0.0 { Sfx::Zap } else { Sfx::Hurt });
        if self.player.energy <= 0 {
            self.player.act = Act::Dead;
            self.player.act_t = 0.0;
            self.dead_t = 0.0;
            self.lives -= 1;
            self.player.vx = 0.0;
        } else {
            self.player.act = Act::Hurt;
            self.player.act_t = 0.0;
        }
    }

    fn hazards(&mut self, _dt: f32) {
        if !level::elec_on(self.time) {
            return;
        }
        let room = level::room(self.level, self.room);
        if self.player.act != Act::Dead && self.player.shock <= 0.0 && level::on_electric(room, self.player.x, self.player.y) {
            self.player.shock = 0.7;
            let dir = if self.player.x < 320.0 { -1 } else { 1 };
            self.damage_player(-dir);
            self.player.vy = -260.0;
        }
        let room_i = self.room;
        let mut shocked = Vec::new();
        for which in 0..2 {
            let e = enemy_ref(&self.runs[room_i], which);
            if e.hurtable() && e.shock <= 0.0 && level::on_electric(room, e.x, e.y) {
                shocked.push(which);
            }
        }
        for which in shocked {
            enemy_mut(&mut self.runs[room_i], which).shock = 0.6;
            self.hurt_enemy(which, 50, false);
            enemy_mut(&mut self.runs[room_i], which).vy = -220.0;
        }
    }

    fn try_exit(&mut self) {
        if self.player.act == Act::Dead || self.slide.is_some() {
            return;
        }
        let flags = self.lantern_flags();
        if self.room == 1 && level::hatch_open(&flags) {
            if self.player.x > 256.0 && self.player.x < 368.0 && self.player.y > 310.0 {
                self.descend();
                return;
            }
        }
        if level::can_exit_right(self.room, &flags) && self.player.x > 618.0 {
            self.slide = Some(Slide { from: self.room, to: self.room + 1, dir: 1, t: 0.0 });
            return;
        }
        if level::can_exit_left(self.room, &flags) && self.player.x < 22.0 {
            self.slide = Some(Slide { from: self.room, to: self.room - 1, dir: -1, t: 0.0 });
            return;
        }
        if self.slide.is_none() {
            self.player.x = self.player.x.clamp(14.0, 626.0);
        }
        if self.player.y > 420.0 {
            self.player.energy = 0;
            self.damage_player(1);
            self.player.y = 200.0;
        }
    }

    fn update_stars(&mut self, dt: f32) {
        let ducking = self.player.h < 30.0;
        let player_box = self.player.aabb();
        let yamo_box = self.runs[self.room].yamo.aabb();
        let yamo_live = self.runs[self.room].yamo.hurtable();
        let mut player_dir: Option<i32> = None;
        let mut yamo_hit = false;
        for star in &mut self.stars {
            star.x += star.vx * dt;
            star.life -= dt;
            star.spin += dt * 14.0;
            if star.life <= 0.0 {
                continue;
            }
            let body = level::Rect { x: star.x - 5.0, y: star.y - 5.0, w: 10.0, h: 10.0 };
            if player_dir.is_none()
                && self.player.act != Act::Dead
                && self.player.invuln <= 0.0
                && !ducking
                && body.overlaps(player_box)
            {
                player_dir = Some(if self.player.x < star.x { -1 } else { 1 });
                star.life = 0.0;
            } else if yamo_live && !yamo_hit && body.overlaps(yamo_box) {
                yamo_hit = true;
                star.life = 0.0;
            }
        }
        self.stars.retain(|star| star.life > 0.0 && star.x > -24.0 && star.x < 664.0);
        if let Some(dir) = player_dir {
            self.damage_player(dir);
        }
        if yamo_hit {
            self.hurt_enemy(1, 50, false);
        }
    }

    /// Drop through an open hatch. The last level clears the fortress.
    /// Score, lives, and chi carry into the next level.
    fn descend(&mut self) {
        self.score += 2000;
        let energy = self.player.energy;
        if self.level + 1 >= level::LEVEL_COUNT {
            self.mode = Mode::Clear;
            self.cue(Sfx::Open);
            return;
        }
        self.level += 1;
        self.room = 0;
        self.slide = None;
        self.stars.clear();
        self.particles.clear();
        self.floaters.clear();
        self.visited = [true, false, false];
        self.runs = RoomRun::set(self.level);
        let spawn = level::room(self.level, 0).spawn_left;
        self.player = Fighter::hero(spawn);
        self.player.energy = energy;
        self.player.invuln = 1.6;
        let name = level::fortress(self.level).name;
        self.banner(&format!("{name} +2000"));
        self.float_at(spawn.0, spawn.1 - 70.0, "+2000", 0xE6B43A);
        self.cue(Sfx::Open);
    }

    fn respawn(&mut self) {
        let spawn = level::room(self.level, self.room).spawn_left;
        self.player = Fighter::hero(spawn);
        self.player.invuln = 1.6;
        self.player.energy = 4;
        self.dead_t = 0.0;
        let run = &mut self.runs[self.room];
        let nsp = level::room(self.level, self.room).ninja;
        let ysp = level::room(self.level, self.room).yamo;
        run.ninja.park(nsp, 1.2);
        run.yamo.park(ysp, 2.4);
        self.stars.clear();
    }

    fn banner(&mut self, s: &str) {
        self.banner = s.to_string();
        self.banner_t = 2.4;
    }

    fn float_at(&mut self, x: f32, y: f32, text: &str, color: u32) {
        self.floaters.push(Floater { x, y, text: text.to_string(), life: 0.9, color });
    }

    fn burst(&mut self, x: f32, y: f32, color: u32) {
        for i in 0..8 {
            let a = i as f32 * 0.8;
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * 40.0,
                vy: a.sin() * 40.0 - 20.0,
                life: 0.4,
                color,
            });
        }
        if self.particles.len() > 160 {
            let extra = self.particles.len() - 160;
            self.particles.drain(0..extra);
        }
    }

    pub fn draw(&self, assets: &Assets, frame: &mut Frame) {
        frame.clear(0x100C18);
        let (sx, sy) = shake_of(self.shake, self.time);
        if let Some(slide) = &self.slide {
            let off = (slide.t * level::VIEW_W as f32) as i32;
            if slide.dir > 0 {
                self.draw_room(assets, frame, slide.from, -off + sx, sy, true);
                self.draw_room(assets, frame, slide.to, level::VIEW_W - off + sx, sy, false);
            } else {
                self.draw_room(assets, frame, slide.from, off + sx, sy, true);
                self.draw_room(assets, frame, slide.to, off - level::VIEW_W + sx, sy, false);
            }
        } else if self.mode == Mode::Title {
            self.draw_room(assets, frame, 0, sx, sy, false);
            self.draw_title_hero(assets, frame);
        } else {
            self.draw_room(assets, frame, self.room, sx, sy, true);
        }
        self.draw_hud(frame);
        if self.mode == Mode::Title {
            self.draw_title(frame);
        } else if self.mode == Mode::Pause {
            gfx::blend_rect(frame, 0, level::HUD, level::VIEW_W, level::VIEW_H, 0x0C0814, 140);
            center_text(frame, "PAUSED", 160, 0xF3E6C4, 3);
        } else if self.mode == Mode::Clear {
            gfx::blend_rect(frame, 0, level::HUD, level::VIEW_W, level::VIEW_H, 0x0C0814, 150);
            center_text(frame, "FORTRESS CLEAR", 120, 0xFFD060, 3);
            center_text(frame, "THE LAST HALL OPENS.", 160, 0xF3E6C4, 1);
            center_text(frame, "THE FORTRESS IS YOURS.", 176, 0xF3E6C4, 1);
            let sc = format!("SCORE {}", self.score);
            center_text(frame, &sc, 204, 0xE6B43A, 2);
            if (self.time * 2.0) as i32 % 2 == 0 {
                center_text(frame, "ENTER TO PLAY AGAIN", 250, 0xF3E6C4, 1);
            }
        } else if self.mode == Mode::Over {
            gfx::blend_rect(frame, 0, level::HUD, level::VIEW_W, level::VIEW_H, 0x140810, 160);
            center_text(frame, "GAME OVER", 140, 0xE05048, 3);
            let sc = format!("SCORE {}", self.score);
            center_text(frame, &sc, 190, 0xF3E6C4, 2);
            if (self.time * 2.0) as i32 % 2 == 0 {
                center_text(frame, "ENTER TO TRY AGAIN", 240, 0xF3E6C4, 1);
            }
        }
    }

    fn draw_title_hero(&self, assets: &Assets, frame: &mut Frame) {
        let feet_y = 304 + level::HUD;
        gfx::blit_frame(frame, &assets.hero, 8, 64, 64, 22, 150, feet_y, (32, 62), false, false);
        gfx::blit_frame(frame, &assets.hero, 8, 64, 64, 0, 420, feet_y, (32, 62), true, false);
    }

    fn draw_title(&self, frame: &mut Frame) {
        // Sky band under the HUD. Keeps the wordmark off the vine and the fighters.
        gfx::blend_rect(frame, 0, level::HUD, level::SCREEN_W, 132, 0x100C18, 176);
        center_text(frame, "BRUCE LEE", 30, 0xFFD56A, 5);
        center_text(frame, "FOUR LEVELS", 70, 0xF3E6C4, 2);
        if (self.time * 2.0) as i32 % 2 == 0 {
            center_text(frame, "ENTER TO BEGIN", 106, 0xF3E6C4, 2);
        }
        center_text(frame, "ARROWS MOVE    UP CLIMB    SPACE JUMP", 128, 0xC8B89A, 1);
        center_text(frame, "Z PUNCH OR KICK    DOWN DUCK    P PAUSE", 140, 0xC8B89A, 1);
    }

    fn draw_hud(&self, frame: &mut Frame) {
        gfx::rect(frame, 0, 0, level::SCREEN_W, level::HUD, 0x16101C);
        gfx::rect(frame, 0, level::HUD - 1, level::SCREEN_W, 1, 0xE6B43A);
        let score = format!("SCORE {:>6}", self.score);
        gfx::draw_text(frame, &score, 8, 8, 0xF3E6C4, 1);
        if self.mode != Mode::Title {
            let lv = format!("LV{}", self.level + 1);
            gfx::draw_text(frame, &lv, 148, 8, 0xC8B89A, 1);
            let name = level::room(self.level, self.room).name;
            let nw = gfx::text_width(name, 1);
            gfx::draw_text(frame, name, (level::SCREEN_W - nw) / 2, 8, 0xE6B43A, 1);
            let got = self.runs[self.room].lanterns.iter().filter(|t| **t).count();
            let lamps = format!("LANTERNS {got}/6");
            gfx::draw_text(frame, &lamps, 408, 8, 0xFFD060, 1);
            gfx::draw_text(frame, "CHI", 548, 8, 0xC8B89A, 1);
            for i in 0..4 {
                let c = if i < self.player.energy { 0xE6B43A } else { 0x3A3040 };
                gfx::rect(frame, 572 + i * 8, 8, 6, 7, c);
            }
            let lives = format!("X{}", self.lives.max(0));
            gfx::draw_text(frame, &lives, 608, 8, 0xF3E6C4, 1);
        }
    }

    fn draw_room(&self, assets: &Assets, frame: &mut Frame, index: usize, ox: i32, oy: i32, with_player: bool) {
        let room = level::room(self.level, index);
        let run = &self.runs[index];
        let flags = self.lantern_flags();
        let hatch = level::hatch_open(&flags);
        let cam_y = level::HUD + oy;
        // Background. Later levels reuse the three paintings with a tint.
        blit_bg(frame, &assets.bg[index], ox, cam_y, level::fortress(self.level).tint);
        // Tiles.
        for r in 0..level::ROWS {
            for c in 0..level::COLS {
                let tile = room.tile(c, r, hatch && index == 1);
                let dx = ox + c * level::TILE;
                let dy = cam_y + r * level::TILE;
                let src = match tile {
                    level::Tile::Empty | level::Tile::Ladder | level::Tile::Vine => continue,
                    level::Tile::OneWay => 2,
                    level::Tile::Electric => {
                        if level::elec_on(self.time) {
                            7
                        } else if level::elec_warn(self.time) && ((self.time * 14.0) as i32) % 2 == 0 {
                            6
                        } else {
                            5
                        }
                    }
                    level::Tile::Solid => {
                        let exposed = r == 0 || !room.tile(c, r - 1, hatch && index == 1).is_full();
                        if !exposed {
                            1
                        } else if index == 0 {
                            0
                        } else {
                            8
                        }
                    }
                };
                gfx::blit(frame, &assets.tiles, src * 16, 0, 16, 16, dx, dy, false, false);
            }
        }
        for r in 0..level::ROWS {
            for c in 0..level::COLS {
                let tile = room.tile(c, r, false);
                let src = match tile {
                    level::Tile::Ladder => 3,
                    level::Tile::Vine => 4,
                    _ => continue,
                };
                gfx::blit(frame, &assets.tiles, src * 16, 0, 16, 16, ox + c * level::TILE, cam_y + r * level::TILE, false, false);
            }
        }
        if index == 1 {
            let hx = ox + 16 * 16 + 56;
            let hy = cam_y + 19 * 16 + 16;
            let frame_i = if hatch { 1 } else { 0 };
            gfx::blit(frame, &assets.hatch, frame_i * 112, 0, 112, 36, hx - 56, hy - 28, false, false);
            if hatch {
                gfx::glow(frame, hx, hy, 28, 0xFFB040);
            }
        }
        // Torches and statue.
        let tframe = ((self.time * 10.0) as i32).rem_euclid(4);
        for &(tx, ty) in room.torches {
            gfx::glow(frame, ox + tx as i32, cam_y + ty as i32 - 22, 16, 0xFF9820);
            gfx::blit(frame, &assets.torch, tframe * 16, 0, 16, 32, ox + tx as i32 - 8, cam_y + ty as i32 - 32, false, false);
        }
        if let Some((sx, sy)) = room.statue {
            gfx::blit(frame, &assets.statue, 0, 0, 64, 72, ox + sx as i32 - 32, cam_y + sy as i32 - 70, false, false);
        }
        // Doors.
        let open_right = level::can_exit_right(index, &flags);
        let open_left = level::can_exit_left(index, &flags);
        if room.door_right {
            let fi = if open_right { 1 } else { 0 };
            gfx::blit(frame, &assets.gate, fi * 48, 0, 48, 88, ox + 616 - 24, cam_y + 304 - 88, false, false);
        }
        if room.door_left {
            let fi = if open_left { 1 } else { 0 };
            gfx::blit(frame, &assets.gate, fi * 48, 0, 48, 88, ox + 16 - 24, cam_y + 304 - 88, false, false);
        }
        // Lanterns.
        let lframe = ((self.time * 8.0) as i32).rem_euclid(4);
        for (i, &(lx, ly)) in room.lanterns.iter().enumerate() {
            if run.lanterns[i] {
                continue;
            }
            gfx::glow(frame, ox + lx as i32, cam_y + ly as i32, 18, 0xFFB040);
            gfx::blit(frame, &assets.lantern, lframe * 32, 0, 32, 40, ox + lx as i32 - 16, cam_y + ly as i32 - 18, false, false);
        }
        if level::elec_on(self.time) && index == 2 {
            gfx::glow(frame, ox + 320, cam_y + 300, 50, 0x50C8FF);
        }
        // Enemies.
        if run.woke {
            draw_enemy(frame, assets, &run.yamo, ox, cam_y, self.time);
            draw_enemy(frame, assets, &run.ninja, ox, cam_y, self.time);
        }
        if index == self.room {
            for star in &self.stars {
                draw_star(frame, ox + star.x as i32, cam_y + star.y as i32, star.spin);
            }
        }
        if with_player && self.mode != Mode::Title {
            draw_player(frame, assets, &self.player, ox, cam_y, self.time);
        }
        // Particles in this room only when it's the active one.
        if index == self.room {
            for p in &self.particles {
                let a = (p.life * 2.0) as i32;
                if a <= 0 {
                    continue;
                }
                gfx::rect(frame, ox + p.x as i32, cam_y + p.y as i32, 2, 2, p.color);
            }
            for f in &self.floaters {
                gfx::draw_text(frame, &f.text, ox + f.x as i32, cam_y + f.y as i32, f.color, 1);
            }
            if self.banner_t > 0.0 {
                center_text(frame, &self.banner, cam_y + 36, 0xFFD060, 2);
            }
            if self.hint > 0.0 && index == 0 && self.mode != Mode::Title {
                let a_note = "ARROWS MOVE   SPACE JUMP   UP CLIMB   Z ATTACK   DOWN DUCK";
                center_text(frame, a_note, cam_y + 8, 0xF3E6C4, 1);
            }
        }
        if self.show_hit && with_player {
            let b = self.player.aabb();
            gfx::rect(frame, ox + b.x as i32, cam_y + b.y as i32, b.w as i32, 1, 0xFF4040);
        }
    }
}

fn draw_player(frame: &mut Frame, assets: &Assets, p: &Fighter, ox: i32, oy: i32, time: f32) {
    if p.invuln > 0.0 && ((time * 18.0) as i32) % 2 == 0 && p.act != Act::Dead {
        return;
    }
    let white = p.act == Act::Hurt && ((time * 24.0) as i32) % 2 == 0;
    let idx = match p.act {
        Act::Idle => {
            if ((p.anim * 2.2) as i32) % 2 == 0 {
                0
            } else {
                1
            }
        }
        Act::Run => 2 + ((p.anim * 12.0) as i32).rem_euclid(6),
        Act::Jump => 8,
        Act::Fall => 9,
        Act::Duck => 10,
        Act::Climb => 11 + ((p.anim * 6.0) as i32).rem_euclid(2),
        Act::Punch => {
            if p.act_t < 0.08 {
                13
            } else if p.act_t < 0.18 {
                14
            } else {
                15
            }
        }
        Act::Kick => {
            if p.grounded {
                if p.act_t < 0.08 {
                    16
                } else if p.act_t < 0.24 {
                    17
                } else {
                    19
                }
            } else if p.act_t < 0.06 {
                16
            } else {
                18
            }
        },
        Act::Hurt => 20,
        Act::Dead => 21,
    };
    shadow(frame, ox + p.x as i32, oy + p.y as i32, 18);
    gfx::blit_frame(frame, &assets.hero, 8, 64, 64, idx, ox + p.x as i32, oy + p.y as i32, (32, 62), p.facing < 0, white);
}

fn draw_enemy(frame: &mut Frame, assets: &Assets, e: &Enemy, ox: i32, oy: i32, time: f32) {
    if e.mode == EMode::Dormant {
        return;
    }
    if e.mode == EMode::Appear && ((time * 16.0) as i32) % 2 == 0 {
        return;
    }
    let white = e.mode == EMode::Hurt && ((time * 24.0) as i32) % 2 == 0;
    let moving = e.vx.abs() > 8.0 && e.grounded;
    if e.kind == EKind::Ninja {
        let idx = match e.mode {
            EMode::Attack => 5 + (((e.mode_t / 0.32) * 3.0) as i32).min(2),
            EMode::Throw => 12 + (((e.mode_t / 0.42) * 3.0) as i32).min(2),
            EMode::Hurt => 8,
            EMode::Dead => 9,
            EMode::Chase if e.climbing => 10 + ((e.anim * 6.0) as i32).rem_euclid(2),
            EMode::Chase if moving => 1 + ((e.anim * 10.0) as i32).rem_euclid(4),
            _ => 0,
        };
        shadow(frame, ox + e.x as i32, oy + e.y as i32, 16);
        gfx::blit_frame(frame, &assets.ninja, 6, 64, 64, idx, ox + e.x as i32, oy + e.y as i32, (32, 61), e.facing < 0, white);
    } else {
        let idx = match e.mode {
            EMode::Attack => 5 + (((e.mode_t / 0.40) * 3.0) as i32).min(2),
            EMode::Hurt => 8,
            EMode::Dead => 9,
            EMode::Chase if moving => 1 + ((e.anim * 8.0) as i32).rem_euclid(4),
            _ => 0,
        };
        shadow(frame, ox + e.x as i32, oy + e.y as i32, 26);
        gfx::blit_frame(frame, &assets.yamo, 5, 96, 96, idx, ox + e.x as i32, oy + e.y as i32, (48, 91), e.facing < 0, white);
    }
}

fn draw_star(frame: &mut Frame, x: i32, y: i32, spin: f32) {
    let turned = (spin as i32) % 2 == 1;
    let arms: [(i32, i32); 8] = if turned {
        [(2, -2), (3, -3), (-2, -2), (-3, -3), (2, 2), (3, 3), (-2, 2), (-3, 3)]
    } else {
        [(0, -3), (0, -4), (0, 3), (0, 4), (-3, 0), (-4, 0), (3, 0), (4, 0)]
    };
    for (dx, dy) in arms {
        frame.pixel(x + dx, y + dy, 0xC8D0DC);
    }
    frame.pixel(x, y, 0xFFE38A);
    frame.pixel(x + 1, y, 0xF2F6FA);
    frame.pixel(x, y - 1, 0xF2F6FA);
    frame.pixel(x - 1, y, 0x8A94A4);
    frame.pixel(x, y + 1, 0x8A94A4);
}

fn shadow(frame: &mut Frame, x: i32, y: i32, half: i32) {
    for i in -half..=half {
        if (i + y) & 1 == 0 {
            frame.pixel(x + i, y, 0x0A0810);
            frame.pixel(x + i, y + 1, 0x0A0810);
        }
    }
}

fn blit_bg(frame: &mut Frame, img: &gfx::Image, ox: i32, oy: i32, tint: (u8, u8, u8)) {
    if tint == (255, 255, 255) {
        gfx::blit(frame, img, 0, 0, img.w, img.h, ox, oy, false, false);
        return;
    }
    let (tr, tg, tb) = tint;
    for y in 0..img.h {
        let dy = oy + y;
        if dy < 0 || dy >= frame.h {
            continue;
        }
        for x in 0..img.w {
            let dx = ox + x;
            if dx < 0 || dx >= frame.w {
                continue;
            }
            let i = ((y * img.w + x) * 4) as usize;
            if img.px[i + 3] < 16 {
                continue;
            }
            let r = img.px[i] as u32 * tr as u32 / 255;
            let g = img.px[i + 1] as u32 * tg as u32 / 255;
            let b = img.px[i + 2] as u32 * tb as u32 / 255;
            frame.px[(dy * frame.w + dx) as usize] = (r << 16) | (g << 8) | b;
        }
    }
}

fn center_text(frame: &mut Frame, s: &str, y: i32, color: u32, scale: i32) {
    let w = gfx::text_width(s, scale);
    gfx::draw_text_shadow(frame, s, (level::SCREEN_W - w) / 2, y, color, 0x140C10, scale);
}

fn shake_of(shake: f32, time: f32) -> (i32, i32) {
    if shake <= 0.0 {
        return (0, 0);
    }
    let m = (shake * 6.0) as i32;
    let x = ((time * 47.0).sin() * m as f32) as i32;
    let y = ((time * 63.0).cos() * m as f32) as i32;
    (x, y)
}

fn tick_fx(particles: &mut Vec<Particle>, floaters: &mut Vec<Floater>, dt: f32) {
    for p in particles.iter_mut() {
        p.life -= dt;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.vy += 80.0 * dt;
    }
    particles.retain(|p| p.life > 0.0);
    for f in floaters.iter_mut() {
        f.life -= dt;
        f.y -= 28.0 * dt;
    }
    floaters.retain(|f| f.life > 0.0);
}

fn surface_near(room: &Room, x: f32, feet: f32) -> Option<f32> {
    let c = (x / 16.0).floor() as i32;
    for r in 0..level::ROWS {
        let t = room.tile(c, r, false);
        if !(t.is_full() || t == level::Tile::OneWay) {
            continue;
        }
        if r > 0 && room.tile(c, r - 1, false).is_full() {
            continue;
        }
        let top = r as f32 * 16.0;
        if (feet - top).abs() <= 18.0 {
            return Some(top);
        }
    }
    None
}

fn solid_point(room: &Room, x: f32, y: f32, hatch: bool) -> bool {
    let c = (x / 16.0).floor() as i32;
    let r = (y / 16.0).floor() as i32;
    room.tile(c, r, hatch).is_full()
}

fn physics_enemy(e: &mut Enemy, room: &Room, doors: &[level::Rect], hatch: bool, dt: f32) {
    let h = if e.kind == EKind::Yamo { 48.0 } else { 40.0 };
    let w = if e.kind == EKind::Yamo { 26.0 } else { 16.0 };
    let mut body = Body { x: e.x, y: e.y, w, h, vx: e.vx, vy: e.vy };
    e.grounded = level::step_body(&mut body, room, true, doors, hatch, dt);
    e.x = body.x;
    e.y = body.y;
    e.vx = body.vx;
    e.vy = body.vy;
}

fn enemy_mut(run: &mut RoomRun, which: usize) -> &mut Enemy {
    if which == 0 {
        &mut run.ninja
    } else {
        &mut run.yamo
    }
}

fn enemy_ref(run: &RoomRun, which: usize) -> &Enemy {
    if which == 0 {
        &run.ninja
    } else {
        &run.yamo
    }
}

impl Fighter {
    fn hero(spawn: (f32, f32)) -> Self {
        Self {
            x: spawn.0,
            y: spawn.1,
            vx: 0.0,
            vy: 0.0,
            w: 16.0,
            h: 42.0,
            facing: 1,
            act: Act::Idle,
            act_t: 0.0,
            grounded: true,
            coyote: 0.0,
            climbing: false,
            invuln: 0.0,
            anim: 0.0,
            attack_hit: false,
            energy: 4,
            shock: 0.0,
        }
    }

    fn body(&self) -> Body {
        Body { x: self.x, y: self.y, w: self.w, h: self.h, vx: self.vx, vy: self.vy }
    }

    fn copy_body(&mut self, b: &Body) {
        self.x = b.x;
        self.y = b.y;
        self.vx = b.vx;
        self.vy = b.vy;
    }

    fn aabb(&self) -> level::Rect {
        level::Rect { x: self.x - self.w * 0.5, y: self.y - self.h, w: self.w, h: self.h }
    }

    fn attack_box(&self) -> Option<level::Rect> {
        let active = match self.act {
            Act::Punch => self.act_t >= 0.07 && self.act_t <= 0.18,
            Act::Kick => self.act_t >= 0.08 && self.act_t <= 0.24,
            _ => false,
        };
        if !active {
            return None;
        }
        let reach = if self.act == Act::Kick { 30.0 } else { 22.0 };
        let height = 14.0;
        let x = if self.facing > 0 { self.x + 4.0 } else { self.x - 4.0 - reach };
        Some(level::Rect { x, y: self.y - 30.0, w: reach, h: height })
    }
}

impl Enemy {
    fn new(kind: EKind, spawn: (f32, f32), delay: f32) -> Self {
        let (hp, _w) = match kind {
            EKind::Ninja => (2, 16.0),
            EKind::Yamo => (3, 26.0),
        };
        Self {
            kind,
            x: spawn.0,
            y: spawn.1,
            vx: 0.0,
            vy: 0.0,
            facing: -1,
            hp,
            max_hp: hp,
            grounded: true,
            climbing: false,
            mode: EMode::Dormant,
            mode_t: delay,
            anim: 0.0,
            cd: 0.0,
            hit: false,
            shock: 0.0,
        }
    }

    fn park(&mut self, spawn: (f32, f32), delay: f32) {
        self.x = spawn.0;
        self.y = spawn.1;
        self.vx = 0.0;
        self.vy = 0.0;
        self.hp = self.max_hp;
        self.mode = EMode::Dormant;
        self.mode_t = delay;
        self.climbing = false;
    }

    fn alive_body(&self) -> bool {
        matches!(self.mode, EMode::Chase | EMode::Attack | EMode::Throw | EMode::Hurt | EMode::Appear)
    }

    fn hurtable(&self) -> bool {
        matches!(self.mode, EMode::Chase | EMode::Attack | EMode::Throw | EMode::Hurt | EMode::Appear)
    }

    fn aabb(&self) -> level::Rect {
        let w = if self.kind == EKind::Yamo { 26.0 } else { 16.0 };
        let h = if self.kind == EKind::Yamo { 48.0 } else { 40.0 };
        level::Rect { x: self.x - w * 0.5, y: self.y - h, w, h }
    }

    fn attack_box(&self) -> Option<level::Rect> {
        let (start, end, reach, yoff) = match self.kind {
            EKind::Ninja => (0.08, 0.22, 36.0, 34.0),
            EKind::Yamo => (0.10, 0.26, 32.0, 20.0),
        };
        if self.mode_t < start || self.mode_t > end {
            return None;
        }
        let x = if self.facing > 0 { self.x } else { self.x - reach };
        Some(level::Rect { x, y: self.y - yoff, w: reach, h: 12.0 })
    }
}

impl RoomRun {
    fn new(level: usize, index: usize, ninja_delay: f32, yamo_delay: f32, woke: bool) -> Self {
        let room = level::room(level, index);
        Self {
            lanterns: [false; 6],
            ninja: Enemy::new(EKind::Ninja, room.ninja, ninja_delay),
            yamo: Enemy::new(EKind::Yamo, room.yamo, yamo_delay),
            woke,
        }
    }

    fn set(level: usize) -> [RoomRun; 3] {
        [
            RoomRun::new(level, 0, 2.2, 5.0, true),
            RoomRun::new(level, 1, 1.6, 3.6, false),
            RoomRun::new(level, 2, 1.4, 3.2, false),
        ]
    }
}

impl Game {
    fn pose_level(&mut self, level: usize) {
        self.mode = Mode::Play;
        self.level = level;
        self.room = 0;
        self.runs = RoomRun::set(level);
        self.runs[0].woke = true;
        let spawn = level::room(level, 0).spawn_left;
        self.player.x = spawn.0 + 48.0;
        self.player.y = spawn.1;
        self.player.act = Act::Idle;
        self.hint = 0.0;
        self.time = 0.45;
    }

    /// Pose the scene for stills. Does not run the simulation.
    pub fn stage(&mut self, scene: &str) {
        *self = Self::new();
        self.mode = Mode::Play;
        match scene {
            "title" => {
                self.mode = Mode::Title;
                self.time = 0.25;
            }
            "court" => {
                self.time = 0.4;
                self.player.x = 150.0;
                self.player.act = Act::Idle;
            }
            "fight" => {
                self.time = 1.1;
                self.player.x = 300.0;
                self.player.y = 250.0;
                self.player.grounded = false;
                self.player.facing = 1;
                self.player.act = Act::Kick;
                self.player.act_t = 0.18;
                self.runs[0].woke = true;
                self.runs[0].ninja.mode = EMode::Attack;
                self.runs[0].ninja.mode_t = 0.16;
                self.runs[0].ninja.x = 420.0;
                self.runs[0].ninja.y = 304.0;
                self.runs[0].ninja.facing = -1;
                self.runs[0].yamo.mode = EMode::Chase;
                self.runs[0].yamo.x = 520.0;
                self.runs[0].yamo.y = 304.0;
                self.runs[0].yamo.facing = -1;
                self.stars.push(Star { x: 340.0, y: 274.0, vx: -180.0, life: 1.0, spin: 1.0 });
                self.runs[0].lanterns[0] = true;
                self.score = 125;
            }
            "hall" => {
                self.room = 1;
                self.time = 0.8;
                self.player.x = 200.0;
                self.player.y = 160.0;
                self.player.act = Act::Climb;
                self.player.climbing = true;
                self.runs[1].woke = true;
                self.runs[1].yamo.mode = EMode::Chase;
                self.runs[1].yamo.x = 360.0;
                self.runs[1].yamo.y = 304.0;
                self.runs[1].ninja.mode = EMode::Chase;
                self.runs[1].ninja.x = 480.0;
                self.runs[1].ninja.y = 80.0;
                self.score = 2000;
            }
            "gallery" => {
                self.room = 2;
                self.time = 2.0;
                self.player.x = 100.0;
                self.player.facing = 1;
                self.player.act = Act::Idle;
                self.runs[2].woke = true;
                self.runs[2].ninja.mode = EMode::Chase;
                self.runs[2].ninja.x = 320.0;
                self.runs[2].ninja.y = 304.0;
                self.runs[2].yamo.mode = EMode::Chase;
                self.runs[2].yamo.x = 540.0;
                self.runs[2].yamo.y = 304.0;
                self.runs[2].yamo.facing = -1;
            }
            "keep" => self.pose_level(1),
            "tower" => self.pose_level(2),
            "vault" => self.pose_level(3),
            "clear" => {
                self.mode = Mode::Clear;
                self.level = level::LEVEL_COUNT - 1;
                self.room = 1;
                self.runs = RoomRun::set(self.level);
                self.time = 1.2;
                self.score = 8750;
                for run in &mut self.runs {
                    run.lanterns = [true; 6];
                    run.woke = true;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hatch_advances_until_the_last_level() {
        let mut g = Game::new();
        g.mode = Mode::Play;
        g.room = 1;
        g.player.x = 300.0;
        g.player.y = 330.0;
        g.player.energy = 2;
        g.score = 100;
        g.lives = 3;
        for run in &mut g.runs {
            run.lanterns = [true; 6];
        }
        g.try_exit();
        assert_eq!(g.level, 1);
        assert_eq!(g.room, 0);
        assert_eq!(g.mode, Mode::Play);
        assert_eq!(g.score, 2100);
        assert_eq!(g.lives, 3);
        assert_eq!(g.player.energy, 2);
        assert!(g.runs.iter().all(|r| r.lanterns.iter().all(|t| !*t)));

        g.level = level::LEVEL_COUNT - 1;
        g.room = 1;
        g.player.x = 300.0;
        g.player.y = 330.0;
        for run in &mut g.runs {
            run.lanterns = [true; 6];
        }
        let score = g.score;
        g.try_exit();
        assert_eq!(g.mode, Mode::Clear);
        assert_eq!(g.level, level::LEVEL_COUNT - 1);
        assert_eq!(g.score, score + 2000);
    }
}
