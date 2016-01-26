use ::phi::{Phi, View, ViewAction};
use ::phi::data::{MaybeAlive, Rectangle};
use ::phi::gfx::{AnimatedSprite, AnimatedSpriteDescr, CopySprite, Sprite};
use ::phi::audio as Audio;
use ::sdl2::pixels::Color;
use ::sdl2::render::Renderer;
use ::std::rc::Rc;
use views::shared::BgSet;
use views::bullets as Bullet;

// Constants

/// Pixels traveled by the player's ship every second, when it is moving.
const DEBUG: bool = false;

const PLAYER_PATH: &'static str = "assets/player_vert.png";
const PLAYER_SPEED: f64 = 360.0;
const PLAYER_W: f64 = 43.0;
const PLAYER_H: f64 = 39.0;

const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;

const TRUMP_PATH: &'static str = "assets/8_bit/trump/trump_sprite_800h.png";
const TRUMPS_WIDE: usize = 4;
const TRUMPS_HIGH: usize = 4;
const TRUMPS_TOTAL: usize = 16;
const TRUMP_WIDTH: f64 = 129.75;
const TRUMP_HEIGHT: f64 = 200.0;
const TRUMP_REST_FRAMES: usize = 4;

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
    vel: f64,
    amplitude: f64,
    angular_vel: f64,
    pos_x: f64,
    origin_y: f64,
}


struct Trump {
    sprite: AnimatedSprite,
    rect: Rectangle,
    amplitude: f64,
    angular_vel: f64,
    pos_x: f64,
    origin_y: f64,
}

impl Trump {
    fn factory(phi: &mut Phi) -> TrumpFactory {
        TrumpFactory {
            sprite: AnimatedSprite::with_fps(
                AnimatedSprite::load_frames(phi, AnimatedSpriteDescr {
                    sprite_type: "trump",
                    image_path: TRUMP_PATH,
                    total_frames: TRUMPS_TOTAL,
                    rest_frames: TRUMP_REST_FRAMES - 1,
                    begin_rest: false,
                    frames_high: TRUMPS_HIGH,
                    frames_wide: TRUMPS_WIDE,
                    frame_w: TRUMP_WIDTH,
                    frame_h: TRUMP_HEIGHT,
                }), 1.0, TRUMPS_TOTAL, TRUMP_REST_FRAMES),
        }
    }

    fn update(mut self, phi: &mut Phi, dt: f64) -> Option<Trump> {
        //self.rect.x -= dt * self.vel;
        self.sprite.add_time(dt);
        self.rect();

        if self.sprite.current_time >= 8.0 {
            None
        } else {
            // when current_time == 0.21500000000000008, the scaling part
            // of the trump sprite animation will be finished.
            // at that point, the animation should be limited to the final four
            // sprite frames, since Trump will be all the way in the foreground.
            if self.sprite.current_time >= 0.21500000000000008 && !self.sprite.is_resting {
                let sprites = self.sprite.sprites.clone();
                let mut sprite_vec = Vec::with_capacity(4);
                let sprite_count = sprites.len();
                for (i, s) in sprites.into_iter().enumerate() {
                    if i >= 12 {
                        sprite_vec.push(s.clone());
                    }
                }

                self.sprite.sprites = sprite_vec;
                self.sprite.is_resting = true;

            }

            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            let rendering = self.rect().to_sdl();
            match rendering {
                None => panic!("Unable to render debug Trump!"),
                Some(trump) => phi.renderer.fill_rect(trump),
            }
        }

        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }

    fn rect(&self) -> Rectangle {
        let dy = self.amplitude * f64::sin(self.angular_vel * self.sprite.current_time);
        Rectangle {
            x: self.pos_x,
            y: self.origin_y + dy,
            w: TRUMP_WIDTH,
            h: TRUMP_HEIGHT,
        }
    }
}

struct TrumpFactory {
    sprite: AnimatedSprite,
}

impl TrumpFactory {
    fn random(&self, phi: &mut Phi) -> Trump {
        let (w, h) = phi.output_size();

        let mut sprite = self.sprite.clone();
        let pos_x = ::rand::random::<f64>().abs() * (w - TRUMP_WIDTH);
        let origin_y = h / 2.0 - 20.0;
        sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        Trump {
            sprite: sprite,
            rect: Rectangle {
                w: TRUMP_WIDTH,
                h: TRUMP_HEIGHT,
                x: pos_x,
                y: origin_y,
            },
            amplitude: 15.0,
            angular_vel: 10.0,
            pos_x: pos_x,
            origin_y: origin_y,
        }
    }
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
    trumps: Vec<Trump>,
    trump_factory: TrumpFactory,
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
                    begin_rest: false,
                    rest_frames: EXPLOSIONS_TOTAL,
                }), EXPLOSION_FPS, EXPLOSIONS_TOTAL, EXPLOSIONS_TOTAL),
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
            trumps: vec![],
            trump_factory: Trump::factory(phi),
            //asteroids: vec![],
            //asteroid_factory: Asteroid::factory(phi),
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

        self.trumps = ::std::mem::replace(&mut self.trumps, vec![])
            .into_iter()
            .filter_map(|trump| trump.update(phi, elapsed))
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

        self.trumps = ::std::mem::replace(&mut self.trumps, vec![])
            .into_iter()
            .filter_map(|trump| {
                let mut trump_alive = true;

                for bullet in &mut transition_bullets {
                    if trump.rect().overlaps(bullet.value.rect()) {
                        trump_alive = false;
                        bullet.alive = false;
                    }
                }

                if trump.rect().overlaps(self.player.rect) {
                    trump_alive = false;
                    player_alive = false;
                }

                if trump_alive {
                    Some(trump)
                } else {
                    // Audio::playback_for(phi, EXPLOSION_AUDIO_PATH);
                    self.explosions.push(
                        self.explosion_factory.at_center(
                            trump.rect().center()));
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
            self.trumps.push(self.trump_factory.random(phi));
        }

        // Clear the scene
        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();

        // Render the Backgrounds
        self.bg.back.render(&mut phi.renderer, elapsed);
        //self.bg.middle.render(&mut phi.renderer, elapsed);

        for trump in &self.trumps {
            trump.render(phi);
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
        //self.bg.front.render(&mut phi.renderer, elapsed);

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
                            h: PLAYER_W,
                            w: PLAYER_H,
                            y: PLAYER_W * x as f64,
                            x: PLAYER_H * y as f64,
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
                        y: (phi.output_size().1 - PLAYER_H) + 20.0,
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
        let cannon1_x = self.rect.w / 2.0 + self.rect.x;
        let cannons_y = self.rect.y;
        let cannon2_x = self.rect.x + PLAYER_W;

        Bullet::spawn_bullets(self.cannon, cannon1_x, cannon2_x, cannons_y)
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
            (true, false) => 0.0,
            (false, true) => 0.0,
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
            y: phi.output_size().1 as f64 - PLAYER_H - 20.0,
            w: phi.output_size().0 as f64,
            h: phi.output_size().1 as f64 * 0.1,
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
