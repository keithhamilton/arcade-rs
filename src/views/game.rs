use ::phi::{Phi, View, ViewAction};
use ::phi::data::{MaybeAlive, Rectangle};
use ::phi::gfx::{AnimatedSprite, AnimatedSpriteDescr, CopySprite, Sprite};
use ::phi::audio as Audio;
use ::sdl2::pixels::Color;
use ::sdl2::render::Renderer;
use views::shared::BgSet;
use views::bullets as Bullet;

// Constants

/// Pixels traveled by the player's ship every second, when it is moving.
const DEBUG: bool = false;

const PLAYER_PATH: &'static str = "assets/player.png";
const PLAYER_SPEED: f64 = 180.0;
const PLAYER_W: f64 = 43.0;
const PLAYER_H: f64 = 39.0;

const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;

const EXPLOSION_PATH: &'static str = "assets/explosion.png";
const EXPLOSION_AUDIO_PATH: &'static str = "assets/explosion.wav";
const EXPLOSIONS_WIDE: usize = 5;
const EXPLOSIONS_HIGH: usize = 4;
const EXPLOSIONS_TOTAL: usize = 17;
const EXPLOSION_SIDE: f64 = 96.0;
const EXPLOSION_FPS: f64 = 16.0;
const EXPLOSION_DURATION: f64 = 1.0 / EXPLOSION_FPS * EXPLOSIONS_TOTAL as f64;


/// The different states our ship might be in. In the image, they're ordered
/// from left to right, then from top to bottom.
#[derive(Clone, Copy)]
enum PlayerFrame {
    UpNorm   = 0,
    UpFast   = 1,
    UpSlow   = 2,
    MidNorm  = 3,
    MidFast  = 4,
    MidSlow  = 5,
    DownNorm = 6,
    DownFast = 7,
    DownSlow = 8
}

// ##############################################################
// structs
// ##############################################################
struct Asteroid {
    sprite: AnimatedSprite,
    rect: Rectangle,
    vel: f64
}


struct AsteroidFactory {
    sprite: AnimatedSprite,
}


pub struct Explosion {
    sprite: AnimatedSprite,
    rect: Rectangle,
    alive_since: f64,
}


pub struct ExplosionFactory {
    sprite: AnimatedSprite,
}

pub struct GameView {
    player: Player,
    bullets: Vec<Box<Bullet::Bullet>>,
    asteroids: Vec<Asteroid>,
    asteroid_factory: AsteroidFactory,
    explosions: Vec<Explosion>,
    explosion_factory: ExplosionFactory,
    bg: BgSet,
}


struct Player {
    rect: Rectangle,
    sprites: Vec<Sprite>,
    current: PlayerFrame,
    cannon: Bullet::CannonType,
}


// ##############################################################
// impls
// ##############################################################
impl Asteroid {
    fn factory(phi: &mut Phi) -> AsteroidFactory {
        AsteroidFactory {
            sprite: AnimatedSprite::with_fps(
                AnimatedSprite::load_frames(phi, AnimatedSpriteDescr {
                    sprite_type: "asteroid",
                    image_path: ASTEROID_PATH,
                    total_frames: ASTEROIDS_TOTAL,
                    frames_high: ASTEROIDS_HIGH,
                    frames_wide: ASTEROIDS_WIDE,
                    frame_w: ASTEROID_SIDE,
                    frame_h: ASTEROID_SIDE,
                }), 1.0),
        }
    }

    fn update(mut self, phi: &mut Phi, dt: f64) -> Option<Asteroid> {
        self.rect.x -= dt * self.vel;
        self.sprite.add_time(dt);

        if self.rect.x <= -ASTEROID_SIDE {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            let rendering = self.rect().to_sdl();
            match rendering {
                None => panic!("Unable to render debug asteroid!"),
                Some(asteroid) => phi.renderer.fill_rect(asteroid),
            }
        }

        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }
}


impl AsteroidFactory {
    fn random(&self, phi: &mut Phi) -> Asteroid {
        let (w, h) = phi.output_size();

        let mut sprite = self.sprite.clone();
        sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        Asteroid {
            sprite: sprite,
            rect: Rectangle {
                w: ASTEROID_SIDE,
                h: ASTEROID_SIDE,
                x: w,
                y: ::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
            },
            vel: ::rand::random::<f64>().abs() * 100.0 + 50.0,
        }
    }
}


impl Explosion {
    fn factory(phi: &mut Phi) -> ExplosionFactory {
        ExplosionFactory {
            sprite: AnimatedSprite::with_fps(
                AnimatedSprite::load_frames(phi, AnimatedSpriteDescr {
                    sprite_type: "explosion",
                    image_path: EXPLOSION_PATH,
                    total_frames: EXPLOSIONS_TOTAL,
                    frames_high: EXPLOSIONS_HIGH,
                    frames_wide: EXPLOSIONS_WIDE,
                    frame_w: EXPLOSION_SIDE,
                    frame_h: EXPLOSION_SIDE,
                }), EXPLOSION_FPS),
        }
    }

    fn update(mut self, dt: f64) -> Option<Explosion> {
        self.alive_since += dt;
        self.sprite.add_time(dt);

        if self.alive_since >= EXPLOSION_DURATION {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }
}


impl ExplosionFactory {
    fn at_center(&self, center: (f64, f64)) -> Explosion {
        let mut sprite = self.sprite.clone();

        Explosion {
            sprite: sprite,
            rect: Rectangle::with_size(EXPLOSION_SIDE, EXPLOSION_SIDE)
                .center_at(center),
            alive_since: 0.0,
        }
    }
}


impl GameView {
    /// We temporarily keep this so that we can instanciate `GameView` in
    /// `main` while developing it further.
    #[allow(dead_code)]
    pub fn new(phi: &mut Phi) -> GameView {
        let bg = BgSet::new(&mut phi.renderer);
        GameView::with_backgrounds(phi, bg)
    }

    pub fn with_backgrounds(phi: &mut Phi, bg: BgSet) -> GameView {
        GameView {
            player: Player::new(phi),
            bullets: vec![],
            asteroids: vec![],
            asteroid_factory: Asteroid::factory(phi),
            explosions: vec![],
            explosion_factory: Explosion::factory(phi),
            bg: bg,
        }
    }
}

impl View for GameView {
    fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit {
            return ViewAction::Quit;
        }

        if phi.events.now.key_escape == Some(true) {
            return ViewAction::ChangeView(Box::new(
                ::views::main_menu::MainMenuView::with_backgrounds(
                    phi, self.bg.clone())));
        }

        self.bullets = ::std::mem::replace(&mut self.bullets, vec![])
            .into_iter()
            .filter_map(|bullet| bullet.update(phi, elapsed))
            .collect();

        self.asteroids = ::std::mem::replace(&mut self.asteroids, vec![])
            .into_iter()
            .filter_map(|asteroid| asteroid.update(phi, elapsed))
            .collect();

        self.explosions = ::std::mem::replace(&mut self.explosions, vec![])
            .into_iter()
            .filter_map(|explosion| explosion.update(elapsed))
            .collect();

        let mut player_alive = true;
        let mut transition_bullets: Vec<_> =
            ::std::mem::replace(&mut self.bullets, vec![])
            .into_iter()
            .map(|bullet| MaybeAlive { alive: true, value: bullet })
            .collect();

        self.asteroids = ::std::mem::replace(&mut self.asteroids, vec![])
            .into_iter()
            .filter_map(|asteroid| {
                let mut asteroid_alive = true;

                for bullet in &mut transition_bullets {
                    if asteroid.rect().overlaps(bullet.value.rect()) {
                        asteroid_alive = false;
                        bullet.alive = false;
                    }
                }

                if asteroid.rect().overlaps(self.player.rect) {
                    asteroid_alive = false;
                    player_alive = false;
                }

                if asteroid_alive {
                    Some(asteroid)
                } else {
                    Audio::playback_for(EXPLOSION_AUDIO_PATH);
                    self.explosions.push(
                        self.explosion_factory.at_center(
                            asteroid.rect().center()));
                    None
                }
            })
            .collect();

        self.bullets = transition_bullets.into_iter()
            .filter_map(MaybeAlive::as_option)
            .collect();

        if !player_alive {
            println!("The player's ship has been destroyed!");
        }

        if phi.events.now.key_space == Some(true) {
            self.bullets.append(&mut self.player.spawn_bullets());
        }

        if ::rand::random::<usize>() % 100 == 0 {
            self.asteroids.push(self.asteroid_factory.random(phi));
        }

        // Clear the scene
        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();

        // Render the Backgrounds
        self.bg.back.render(&mut phi.renderer, elapsed);
        self.bg.middle.render(&mut phi.renderer, elapsed);

        for asteroid in &self.asteroids {
            asteroid.render(phi);
        }

        for bullet in &self.bullets {
            bullet.render(phi);
        }

        for explosion in &self.explosions {
            explosion.render(phi);
        }

        self.player.update(phi, elapsed);
        self.player.render(phi);

        // Render the foreground
        self.bg.front.render(&mut phi.renderer, elapsed);

        ViewAction::None
    }
}


impl Player {
    pub fn new(phi: &mut Phi) -> Player {
        let spritesheet = Sprite::load(&mut phi.renderer, PLAYER_PATH);
        match spritesheet {
            None => panic!("Could not render the player!"),
            Some(player) => {
                let mut sprites = Vec::with_capacity(9);
                for y in 0..3 {
                    for x in 0..3 {
                        let rendering = player.region(Rectangle {
                            w: PLAYER_W,
                            h: PLAYER_H,
                            x: PLAYER_W * x as f64,
                            y: PLAYER_H * y as f64,
                        });

                        match rendering {
                            None => panic!("Couldn't render sprite region!"),
                            Some(sprite) => sprites.push(sprite),
                        }
                    }
                }

                Player {
                    rect: Rectangle {
                        x: 64.0,
                        y: (phi.output_size().1 - PLAYER_H) / 2.0,
                        w: PLAYER_W,
                        h: PLAYER_H,
                    },
                    sprites: sprites,
                    current: PlayerFrame::MidNorm,
                    cannon: Bullet::CannonType::RectBullet,
                }
            }
        }
    }

    pub fn render(&self, phi: &mut Phi) {
        // Render the bounding box (for debugging purposes)
        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            let rendering = self.rect.to_sdl();
            match rendering {
                None => panic!("Unable to render ship background!"),
                Some(ship_bg) => phi.renderer.fill_rect(ship_bg),
            }
        }

        // Render the ship
        phi.renderer.copy_sprite(
            &self.sprites[self.current as usize],
            self.rect);

    }

    pub fn spawn_bullets(&self) -> Vec<Box<Bullet::Bullet>> {
        let cannons_x = self.rect.x + 30.0;
        let cannon1_y = self.rect.y + 6.0;
        let cannon2_y = self.rect.y + PLAYER_H - 10.0;

        Bullet::spawn_bullets(self.cannon, cannons_x, cannon1_y, cannon2_y)
    }

    pub fn update(&mut self, phi: &mut Phi, elapsed: f64) {
        if phi.events.now.key_1 == Some(true) {
            self.cannon = Bullet::CannonType::RectBullet;
        }

        if phi.events.now.key_2 == Some(true) {
            self.cannon = Bullet::CannonType::SineBullet {
                amplitude: 10.0,
                angular_vel: 15.0,
            };
        }

        if phi.events.now.key_3 == Some(true) {
            self.cannon = Bullet::CannonType::DivergentBullet {
                a: 100.0,
                b: 1.2,
            };
        }

        // Move the player's ship
        let diagonal =
            (phi.events.key_up ^ phi.events.key_down) &&
            (phi.events.key_left ^ phi.events.key_right);

        let moved =
            if diagonal { 1.0 / 2.0f64.sqrt() }
            else { 1.0 } * PLAYER_SPEED * elapsed;

        let dx = match (phi.events.key_left, phi.events.key_right) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        let dy = match (phi.events.key_up, phi.events.key_down) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        self.rect.x += dx;
        self.rect.y += dy;

        // The movable region spans the entire height of the window and 70% of its
        // width. This way, the player cannot get to the far right of the screen, where
        // we will spawn the asteroids, and get immediately eliminated.
        //
        // We restrain the width because most screens are wider than they are high.
        let movable_region = Rectangle {
            x: 0.0,
            y: 0.0,
            w: phi.output_size().0 as f64 * 0.70,
            h: phi.output_size().1 as f64,
        };

        // If the player cannot fit in the screen, then there is a problem and
        // the game should be promptly aborted.
        self.rect = self.rect.move_inside(movable_region).unwrap();


        // Select the appropriate sprite of the ship to show.
        self.current =
            if dx == 0.0 && dy < 0.0       { PlayerFrame::UpNorm }
            else if dx > 0.0 && dy < 0.0   { PlayerFrame::UpFast }
            else if dx < 0.0 && dy < 0.0   { PlayerFrame::UpSlow }
            else if dx == 0.0 && dy == 0.0 { PlayerFrame::MidNorm }
            else if dx > 0.0 && dy == 0.0  { PlayerFrame::MidFast }
            else if dx < 0.0 && dy == 0.0  { PlayerFrame::MidSlow }
            else if dx == 0.0 && dy > 0.0  { PlayerFrame::DownNorm }
            else if dx > 0.0 && dy > 0.0   { PlayerFrame::DownFast }
            else if dx < 0.0 && dy > 0.0   { PlayerFrame::DownSlow }
            else { unreachable!() };

    }
}
