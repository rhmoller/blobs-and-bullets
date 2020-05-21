use crate::engine::gamepad::TwinStick;
use crate::engine::math::{vec2_distance, Vec2};
use crate::engine::preloader::{Preloader, Resources};
use crate::engine::renderer::CanvasRenderer;
use crate::engine::tiled::TileMap;
use crate::engine::GameContext;
use js_sys::Math::random;
use std::f64::consts::PI;
use web_sys::HtmlImageElement;

const PLAYER_1_LEFT_CYCLE: [u8; 2] = [161, 169];
const PLAYER_1_RIGHT_CYCLE: [u8; 2] = [160, 168];
const PLAYER_2_LEFT_CYCLE: [u8; 2] = [163, 171];
const PLAYER_2_RIGHT_CYCLE: [u8; 2] = [162, 170];

const ENEMY_0_LEFT_CYCLE: [u8; 2] = [121, 113];
const ENEMY_0_RIGHT_CYCLE: [u8; 2] = [120, 112];
const ENEMY_1_LEFT_CYCLE: [u8; 2] = [123, 115];
const ENEMY_1_RIGHT_CYCLE: [u8; 2] = [122, 114];
const ENEMY_2_LEFT_CYCLE: [u8; 2] = [125, 117];
const ENEMY_2_RIGHT_CYCLE: [u8; 2] = [124, 116];

#[derive(Copy, Clone)]
enum Splat {
    Sparks,
    Blood,
    Enemy(u8),
    Water,
    Explosion,
}

pub struct MyGame {
    player_1: Player,
    player_2: Player,

    sprites: Option<HtmlImageElement>,
    numbers: Option<HtmlImageElement>,
    map: Option<TileMap>,
    bullets: Vec<Bullet>,
    splatter: Vec<(Vec2, Splat, Vec2, i8)>,

    spawn_points: Vec<(Vec2, u8, u8)>,
    enemies: Vec<(Vec2, f64, u8)>,
    boss: Option<Boss>,

    power_ups: Vec<(Vec2, i8)>,
}

pub struct Boss {
    pub pos: Vec2,
    pub health: i32,
    pub heat: f64,
    pub charging: bool,
    pub tx: f64,
}

pub struct Player {
    pub number: u8,
    pub pos: Vec2,
    pub aim: Vec2,
    pub face_left: bool,
    pub moving: bool,

    pub shooting: bool,
    pub heat: f64,

    pub score: i32,
    pub next_score: i32,

    pub health: i32,
    pub ammo: i32,
    pub next_ammo: i32,
}

#[derive(Copy, Clone)]
enum Shooter {
    Player(u8),
    Enemy(u8),
    Boss,
}

struct Bullet(Vec2, Vec2, Shooter);

impl Player {
    fn new(pos: Vec2, number: u8) -> Player {
        let aim = Vec2::new(1.0, 0.0);
        let shooting = false;
        let face_left = false;
        let heat = 0.0;

        Player {
            number,
            pos,
            aim,
            face_left,
            moving: false,
            shooting,
            heat,
            score: 0,
            next_score: 0,
            health: 3 * 5,
            ammo: 250,
            next_ammo: 0,
        }
    }

    fn update(
        &mut self,
        gamepad: &TwinStick,
        bullets: &mut Vec<Bullet>,
        splatter: &mut Vec<(Vec2, Splat, Vec2, i8)>,
        map: &TileMap,
    ) {
        let dir = Vec2::new(gamepad.move_x_axis, gamepad.move_y_axis);

        let tiles = &map.layers[0].data;
        let t = get_tile_at(
            tiles,
            self.pos.x + dir.x * 2.0,
            self.pos.y + dir.y * 2.0 + 8.0,
        );
        let is_wall = is_wall_tile(t);
        if (!is_wall) && self.health > 0 {
            self.pos.x += dir.x * 2.0;
            self.pos.y += dir.y * 2.0;
        }

        let is_water_tile = is_water_tile(t);
        if self.moving && is_water_tile && random() < 0.3 {
            MyGame::add_splatter(splatter, &self.pos, 1, Splat::Water);
        }

        if dir.x < -0.02 {
            self.face_left = true;
        } else if dir.x > 0.02 {
            self.face_left = false;
        }
        self.moving = dir.length() > 0.1;
        self.aim.x = gamepad.aim_x_axis;
        self.aim.y = gamepad.aim_y_axis;
        self.shooting = gamepad.shoot;

        let can_shoot = self.health > 0
            && self.heat < 1.0
            && (self.aim.x.abs() > 0.25 || self.aim.y.abs() > 0.25);

        if self.shooting && can_shoot && self.ammo > 0 {
            self.heat += 10.0;
            let mut bullet_velocity = self.aim.clone();
            bullet_velocity.normalize();
            bullet_velocity.scale(7.0);
            bullets.push(Bullet(
                self.pos.clone(),
                bullet_velocity,
                Shooter::Player(self.number),
            ));
            self.ammo -= 1;
        } else if self.heat > 0.0 {
            self.heat -= 1.0;
        }
    }
}

impl MyGame {
    pub fn new() -> Self {
        let spawn_points = vec![
            (Vec2::new(16. * 30., 16. * 19.), 0, 1),
            (Vec2::new(16. * 13., 16. * 23.), 1, 0),
            (Vec2::new(16. * 50., 16. * 24.), 2, 0),
            (Vec2::new(16. * 18., 16. * 8.), 2, 0),
        ];

        MyGame {
            player_1: Player::new(Vec2::new(100., 100.), 1),
            player_2: Player::new(Vec2::new(170., 90.), 2),
            sprites: None,
            numbers: None,
            map: None,
            bullets: Vec::new(),
            splatter: Vec::new(),
            spawn_points,
            enemies: Vec::new(),
            boss: None,
            power_ups: Vec::new(),
        }
    }

    pub fn preload(&self, loader: &mut Preloader) {
        loader.load_image(String::from("assets/lorez.png"));
        loader.load_image(String::from("assets/numbers.png"));
        loader.load_json(String::from("assets/tilemap.json"));
    }

    pub fn init(&mut self, mut resources: Resources) {
        let sprite_sheet: Option<HtmlImageElement> = resources.images.remove("assets/lorez.png");
        if let Some(img) = sprite_sheet {
            self.sprites.replace(img);
        };
        let bitmap_font: Option<HtmlImageElement> = resources.images.remove("assets/numbers.png");
        if let Some(img) = bitmap_font {
            self.numbers.replace(img);
        };
        let tilemap = resources.jsons.remove("assets/tilemap.json");
        if let Some(map) = tilemap {
            let realmap = TileMap::new_from_json(&map);
            self.map.replace(realmap);
        }
    }

    pub fn update(&mut self, ctx: &GameContext) {
        self.player_1.update(
            &ctx.gamepad_1,
            &mut self.bullets,
            &mut self.splatter,
            &self.map.as_ref().unwrap(),
        );
        self.player_2.update(
            &ctx.gamepad_2,
            &mut self.bullets,
            &mut self.splatter,
            &self.map.as_ref().unwrap(),
        );
        self.update_bullets(&ctx);
        self.update_splatter();
        self.spawn_enemies();
        self.update_enemies(&ctx);
        self.update_boss();
        self.update_power_ups();
        self.update_scores();
    }

    fn update_power_ups(&mut self) {
        let player1_pos = self.player_1.pos.clone();
        let player2_pos = self.player_2.pos.clone();
        let mut player_1_ammo = 0;
        let mut player_2_ammo = 0;
        let mut player_1_health = 0;
        let mut player_2_health = 0;
        self.power_ups.retain(|p| {
            let hit1 = vec2_distance(&p.0, &player1_pos) < 16.0;
            if hit1 {
                match p.1 {
                    0 => player_1_ammo += 25,
                    1 => player_1_health += 5,
                    _ => (),
                }
            }

            let hit2 = vec2_distance(&p.0, &player2_pos) < 16.0;
            if hit2 {
                match p.1 {
                    0 => player_2_ammo += 25,
                    1 => player_2_health += 5,
                    _ => (),
                }
            }

            !(hit1 || hit2)
        });
        self.player_1.ammo += player_1_ammo;
        self.player_2.ammo += player_2_ammo;
        self.player_1.health += player_1_health;
        self.player_2.health += player_2_health;
    }

    pub fn update_bullets(&mut self, ctx: &GameContext) {
        for b in self.bullets.iter_mut() {
            b.0.x += b.1.x;
            b.0.y += b.1.y;
        }

        let mut hit_bullets = Vec::new();
        let mut hit_enemies = Vec::new();
        let mut hit_spawn_points = Vec::new();
        let mut hit_boss = 0;

        for (b_idx, b) in self.bullets.iter().enumerate() {
            let inside_bounds =
                b.0.x > 0.0 && b.0.x < ctx.window_width && b.0.y > 0.0 && b.0.y < ctx.window_height;
            if inside_bounds {
                let t = get_tile_at(&self.map.as_ref().unwrap().layers[0].data, b.0.x, b.0.y);
                let hit_wall = is_wall_tile(t);
                if hit_wall {
                    let kind = match b.2 {
                        Shooter::Enemy(t) => Splat::Enemy(t),
                        _ => Splat::Sparks,
                    };
                    hit_bullets.push(b_idx);
                    Self::add_splatter(&mut self.splatter, &b.0, 4, kind);
                } else {
                    match b.2 {
                        Shooter::Player(player_num) => {
                            let hit = self
                                .enemies
                                .iter()
                                .enumerate()
                                .find(|(_e_idx, e)| vec2_distance(&e.0, &b.0) < 16.0);
                            let hit_spawn_point = self
                                .spawn_points
                                .iter()
                                .enumerate()
                                .find(|(_e_idx, e)| vec2_distance(&e.0, &b.0) < 16.0);
                            if let Some((e_idx, e)) = hit {
                                hit_bullets.push(b_idx);
                                hit_enemies.push(e_idx);
                                if player_num == 1 {
                                    self.player_1.next_score += 125;
                                } else {
                                    self.player_2.next_score += 125;
                                }
                                if random() < 0.3 {
                                    self.power_ups
                                        .push((b.0.clone(), if random() < 0.5 { 0 } else { 1 }));
                                }
                                let kind = Splat::Enemy(e.2);
                                MyGame::add_splatter(&mut self.splatter, &b.0, 8, kind);
                            } else if let Some((e_idx, e)) = hit_spawn_point {
                                if e.1 != 3 && e.2 > 0 {
                                    hit_bullets.push(b_idx);
                                    hit_spawn_points.push(e_idx);
                                    if player_num == 1 {
                                        self.player_1.next_score += 125;
                                    } else {
                                        self.player_2.next_score += 125;
                                    }
                                    if e.2 < 2 {
                                        self.power_ups.push((
                                            b.0.clone(),
                                            if random() < 0.5 { 0 } else { 1 },
                                        ));
                                        let kind = Splat::Enemy(e.1);
                                        MyGame::add_splatter(&mut self.splatter, &b.0, 8, kind);
                                        Self::add_splatter(
                                            &mut self.splatter,
                                            &b.0,
                                            16,
                                            Splat::Explosion,
                                        )
                                    }
                                    let kind = Splat::Enemy(e.1);
                                    MyGame::add_splatter(&mut self.splatter, &b.0, 8, kind);
                                }
                            } else if let Some(boss) = &self.boss {
                                let bp = boss.pos.clone();
                                if vec2_distance(&b.0, &bp) < 32.0 {
                                    hit_bullets.push(b_idx);
                                    hit_boss += 1;
                                    if player_num == 1 {
                                        self.player_1.next_score += 255;
                                    } else {
                                        self.player_2.next_score += 250;
                                    }
                                    MyGame::add_splatter(
                                        &mut self.splatter,
                                        &b.0,
                                        8,
                                        Splat::Enemy(0),
                                    );
                                    MyGame::add_splatter(&mut self.splatter, &b.0, 8, Splat::Blood);
                                }
                            }
                        }
                        Shooter::Enemy(t) => {
                            let hit1 = vec2_distance(&self.player_1.pos, &b.0) < 16.0;
                            let hit2 = vec2_distance(&self.player_2.pos, &b.0) < 16.0;
                            if hit1 {
                                self.player_1.health -= 1;
                            }
                            if hit2 {
                                self.player_2.health -= 1;
                            }
                            if hit1 || hit2 {
                                hit_bullets.push(b_idx);
                                MyGame::add_splatter(&mut self.splatter, &b.0, 8, Splat::Blood);
                                MyGame::add_splatter(&mut self.splatter, &b.0, 8, Splat::Enemy(t));
                            }
                        }
                        Shooter::Boss => {
                            let hit1 = vec2_distance(&self.player_1.pos, &b.0) < 16.0;
                            let hit2 = vec2_distance(&self.player_2.pos, &b.0) < 16.0;
                            if hit1 {
                                self.player_1.health -= 3;
                            }
                            if hit2 {
                                self.player_2.health -= 3;
                            }
                            if hit1 || hit2 {
                                hit_bullets.push(b_idx);
                                MyGame::add_splatter(&mut self.splatter, &b.0, 8, Splat::Enemy(0));
                            }
                        }
                    }
                }
            }
        }
        hit_bullets.sort();
        hit_bullets.reverse();
        for idx in hit_bullets.iter() {
            self.bullets.remove(*idx);
        }

        hit_enemies.reverse();
        for idx in hit_enemies.iter() {
            self.enemies.remove(*idx);
        }

        let mut destroyed_spawn_points = false;
        for idx in hit_spawn_points.iter() {
            let sp = self.spawn_points.get_mut(*idx);
            if let Some(p) = sp {
                if p.2 > 0 {
                    p.2 -= 1;
                    if p.2 < 1 {
                        p.1 = 3;
                        destroyed_spawn_points = true;
                    }
                }
            }
        }

        let active_spawn_point = self.spawn_points.iter().find(|sp| sp.2 > 0);
        if destroyed_spawn_points && active_spawn_point.is_none() {
            self.boss.replace(Boss {
                pos: Vec2::new(264., -55.),
                health: 80,
                heat: 100.,
                charging: false,
                tx: 100. + random() * 300.,
            });
        }

        if let Some(boss) = &mut self.boss {
            boss.health -= hit_boss;
            if boss.health < 1 {
                self.boss.take();
            }
        }
    }

    fn add_splatter(
        splatter: &mut Vec<(Vec2, Splat, Vec2, i8)>,
        pos: &Vec2,
        amount: u8,
        kind: Splat,
    ) {
        for i in 0..amount {
            let v = match kind {
                Splat::Explosion => 1.0 + 2.0 * random(),
                _ => 0.5 + 4.0 * random(),
            };
            splatter.push((
                pos.clone(),
                kind,
                Vec2::new(
                    v * f64::cos((i as f64 / amount as f64 + 8.0 * random()) * PI),
                    v * f64::sin((i as f64 / amount as f64 + 8.0 * random()) * PI),
                ),
                match kind {
                    Splat::Explosion => 24 + (8. * random()) as i8,
                    _ => (10. + 5. * random()) as i8,
                },
            ));
        }
    }

    fn update_splatter(&mut self) {
        for s in self.splatter.iter_mut() {
            s.0.add(&s.2);
            s.3 -= 1;
        }
        self.splatter.retain(|s| s.3 > 0);
    }

    fn spawn_enemies(&mut self) {
        let no_boss = self.boss.is_none();
        if random() < 0.02 && no_boss {
            let idx = (4. * random()).floor() as usize;
            let spawn_points = &self.spawn_points[idx];
            if spawn_points.2 > 0 {
                self.enemies.push((
                    Vec2::new(spawn_points.0.x, spawn_points.0.y),
                    random() * PI,
                    spawn_points.1,
                ));
            }
        }
    }

    fn update_scores(&mut self) {
        if self.player_1.score < self.player_1.next_score {
            let delta = self.player_1.next_score - self.player_1.score;
            self.player_1.score += if delta > 50 { 25 } else { 5 };
        }
        if self.player_2.score < self.player_2.next_score {
            let delta = self.player_2.next_score - self.player_2.score;
            self.player_2.score += if delta > 50 { 25 } else { 5 };
        }
    }
    fn update_enemies(&mut self, ctx: &GameContext) {
        for b in self.enemies.iter_mut() {
            let dir = Vec2::new(f64::cos(b.1), f64::sin(b.1));
            let t = get_tile_at(
                &self.map.as_ref().unwrap().layers[0].data,
                b.0.x + dir.x,
                b.0.y + dir.y + 8.0,
            );

            let is_wall = is_wall_tile(t);
            if !is_wall {
                b.0.x += dir.x;
                b.0.y += dir.y;
            }

            if random() < 0.01 {
                let target = if random() < 0.5 {
                    &self.player_1.pos
                } else {
                    &self.player_2.pos
                };
                let mut bullet_velocity = Vec2::new(target.x - b.0.x, target.y - b.0.y);
                bullet_velocity.normalize();
                bullet_velocity.scale(7.0);
                self.bullets
                    .push(Bullet(b.0.clone(), bullet_velocity, Shooter::Enemy(b.2)));
            }
            b.1 += random() * 0.5 - 0.25;
        }
        self.enemies.retain(|b| {
            b.0.x > 0.0 && b.0.x < ctx.window_width && b.0.y > 0.0 && b.0.y < ctx.window_height
        });
    }

    fn update_boss(&mut self) {
        if let Some(boss) = &mut self.boss {
            if boss.pos.y < 205.0 {
                boss.pos.y += 1.;
            } else {
                boss.heat += 1.;
                if !boss.charging {
                    if boss.pos.x < boss.tx - 2. {
                        boss.pos.x += 1.5;
                    } else if boss.pos.x > boss.tx + 2. {
                        boss.pos.x -= 1.5;
                    } else {
                        boss.charging = true;
                        boss.heat = 0.;
                    }
                } else if boss.heat > 25. {
                    boss.tx += 100.0 - 200. * random();
                    if boss.tx < 50. {
                        boss.tx += 50.
                    } else if boss.tx > 7. * 163. {
                        boss.tx -= 20.
                    }

                    boss.charging = false;
                    boss.heat = 0.;
                    let offset = random();
                    for i in 0..10 {
                        let a = offset + (PI / 5.0) * i as f64;
                        let v = 5.;
                        let p = boss.pos.clone();
                        self.bullets.push(Bullet(
                            p,
                            Vec2::new(v * f64::cos(a), v * f64::sin(a)),
                            Shooter::Boss,
                        ));
                    }
                }
            }
        }
    }

    pub fn render(&self, renderer: &CanvasRenderer, ctx: &GameContext) {
        renderer.clear();

        let option = self.sprites.as_ref();
        if let Some(image) = option {
            if let Some(map) = self.map.as_ref() {
                renderer.draw_map(&map, image);
            }

            for sp in self.spawn_points.iter() {
                let idx = 144
                    + sp.1 * 2
                    + if (ctx.tick as u32 + sp.1 as u32 * 20) % 60 > 30 {
                        0
                    } else {
                        8
                    };
                renderer.draw_block(
                    &image,
                    if sp.2 < 1 { 150 } else { idx },
                    sp.0.x - 16.,
                    sp.0.y - 8.,
                    2,
                    1,
                );
            }

            if self.player_1.health > 0 {
                let frame = if self.player_1.moving {
                    (ctx.tick / 4) % 2
                } else {
                    0
                } as usize;
                let sprite = if self.player_1.face_left {
                    PLAYER_1_LEFT_CYCLE[frame]
                } else {
                    PLAYER_1_RIGHT_CYCLE[frame]
                };
                renderer.draw_sprite(
                    image,
                    sprite,
                    self.player_1.pos.x - 8.,
                    self.player_1.pos.y - 8.,
                );
                if self.player_1.aim.length() > 0.1 {
                    renderer.draw_sprite(
                        &image,
                        164,
                        self.player_1.pos.x - 8. + 16. * self.player_1.aim.x,
                        self.player_1.pos.y - 8. + 16. * self.player_1.aim.y,
                    );
                }
            } else {
                renderer.draw_sprite(&image, 167, self.player_1.pos.x, self.player_1.pos.y);
            }

            if self.player_2.health > 0 {
                let frame = if self.player_2.moving {
                    (ctx.tick / 4) % 2
                } else {
                    0
                } as usize;
                let sprite = if self.player_2.face_left {
                    PLAYER_2_LEFT_CYCLE[frame]
                } else {
                    PLAYER_2_RIGHT_CYCLE[frame]
                };
                renderer.draw_sprite(
                    image,
                    sprite,
                    self.player_2.pos.x - 8.,
                    self.player_2.pos.y - 8.,
                );
                if self.player_2.aim.length() > 0.1 {
                    renderer.draw_sprite(
                        &image,
                        164,
                        self.player_2.pos.x - 8. + 16. * self.player_2.aim.x,
                        self.player_2.pos.y - 8. + 16. * self.player_2.aim.y,
                    );
                }
            } else {
                renderer.draw_sprite(&image, 167, self.player_2.pos.x, self.player_2.pos.y);
            }

            for (bi, b) in self.enemies.iter().enumerate() {
                let dx = f64::cos(b.1);
                let frame = ((ctx.tick as u32 + bi as u32) / 5 % 2) as usize;

                let left = dx < 0.;
                let table = match (b.2, left) {
                    (0, true) => ENEMY_0_LEFT_CYCLE,
                    (0, false) => ENEMY_0_RIGHT_CYCLE,
                    (1, true) => ENEMY_1_LEFT_CYCLE,
                    (1, false) => ENEMY_1_RIGHT_CYCLE,
                    (2, true) => ENEMY_2_LEFT_CYCLE,
                    (2, false) => ENEMY_2_RIGHT_CYCLE,
                    _ => ENEMY_0_LEFT_CYCLE,
                };
                let sprite = table[frame];
                renderer.draw_sprite(&image, sprite, b.0.x - 8., b.0.y - 8.);
            }

            if let Some(boss) = &self.boss {
                if boss.charging {
                    renderer.draw_block(&image, 176, boss.pos.x - 12., boss.pos.y - 12., 3, 3)
                } else {
                    renderer.draw_block(&image, 179, boss.pos.x - 12., boss.pos.y - 12., 3, 3)
                }

                renderer.draw_rect("black", boss.pos.x - 12., boss.pos.y + 40., 48., 2.);
                renderer.draw_rect(
                    "red",
                    boss.pos.x - 12.,
                    boss.pos.y + 40.,
                    (48. * boss.health as f64) / 80.0,
                    2.,
                );
            }

            for b in self.bullets.iter() {
                let sprite = match b.2 {
                    Shooter::Player(_) => 165,
                    Shooter::Enemy(t) => match t {
                        1 => 130,
                        2 => 132,
                        _ => 128,
                    },
                    Shooter::Boss => 78,
                };
                renderer.draw_sprite(&image, sprite, b.0.x - 8., b.0.y - 8.);
            }

            for b in self.power_ups.iter() {
                match b.1 {
                    0 => renderer.draw_sprite(&image, 106, b.0.x - 8., b.0.y - 8.),
                    1 => renderer.draw_sprite(&image, 98, b.0.x - 8., b.0.y - 8.),
                    _ => (),
                }
            }

            for b in self.splatter.iter() {
                let frame = if b.3 < 10 { 8 } else { 0 };
                let idx = match b.1 {
                    Splat::Sparks => 134 + frame,
                    Splat::Water => 135 + frame,
                    Splat::Blood => 166 + frame,
                    Splat::Enemy(e) => 129 + e * 2 + frame,
                    Splat::Explosion => 208 + u8::min(7, 8 - (b.3 / 4) as u8),
                };
                renderer.draw_sprite(&image, idx, b.0.x - 8., b.0.y - 8.);
            }

            let numbers = &self.numbers.as_ref().unwrap();

            let mut px = 10.0;
            let score = format!("{}", self.player_1.score);
            renderer.draw_numbers(&numbers, px, 10., &score);
            px += 6.0 * 8.0;

            let ammo = format!("{}", self.player_1.ammo);
            renderer.draw_ammo(&numbers, px, 10.0);
            px += 12.0;
            renderer.draw_numbers(&numbers, px, 10., &ammo);
            px += 5.0 * 10.0;
            renderer.draw_hearts(&numbers, px, 10., self.player_1.health);

            let mut px = ctx.window_width - 5.0 * 12.0;
            renderer.draw_hearts(&numbers, px, 10., self.player_2.health);

            let ammo = format!("{}", self.player_2.ammo);
            px -= 5.0 * 10.0;
            renderer.draw_numbers(&numbers, px, 10., &ammo);
            px -= 12.0;
            renderer.draw_ammo(&numbers, px, 10.0);

            let score = format!("{}", self.player_2.score);
            px -= 6.0 * 8.0;
            renderer.draw_numbers(&numbers, px, 10., &score);

            if self.player_1.health < 1 && self.player_2.health < 1 {
                renderer.draw_rect("#0006", 96., 270. - 48., 1920. / 2. - 192., 96.);
                let text_x = (((1920 / 2) - 13 * 32) / 2) as f64;
                renderer.draw_big_text(
                    &self.numbers.as_ref().unwrap(),
                    text_x,
                    270. - 16.,
                    &"@ GAME OVER @",
                )
            }
        } else {
        }
    }
}

fn get_tile_at(data: &[u8], x: f64, y: f64) -> u8 {
    let tx = (x / 16.0) as usize;
    let ty = (y / 16.0) as usize;
    if tx < 60 && ty < 34 {
        data[tx + ty * 60]
    } else {
        0
    }
}

fn is_wall_tile(t: u8) -> bool {
    t < 23 || t == 56 || t == 57 || t == 58 || t == 69 || t == 70 || t == 71
}

fn is_water_tile(t: u8) -> bool {
    t >= 72 && t <= 93
}
