use phi::Phi;
use phi::data::Rectangle;
use sdl2::pixels::Color;

pub const BULLET_SPEED: f64 = 600.0;
pub const BULLET_SPEED_SLOW: f64 = 300.0;
pub const BULLET_W: f64 = 4.0;
pub const BULLET_H: f64 = 8.0;


#[derive(Clone, Copy)]
pub enum CannonType {
    RectBullet,
    SineBullet { amplitude: f64, angular_vel: f64 },
    DivergentBullet { a: f64, b: f64 },
}

// ##############################################################
// structs
// ##############################################################
pub struct DivergentBullet {
    pos_x: f64,
    origin_y: f64,
    a: f64, // influences the bump's height
    b: f64, // influences the bump's width
    total_time: f64,
}


pub struct RectBullet {
    rect: Rectangle,
}


pub struct SineBullet {
    pos_x: f64,
    origin_y: f64,
    amplitude: f64,
    angular_vel: f64,
    total_time: f64,
}


// ##############################################################
// traits
// ##############################################################
pub trait Bullet {

    /// By using Box<Self> as the type of `self`, we are keeping the data behind
    /// a pointer, but `move` the pointer as update is called.
    fn update(self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>>;

    /// Here we take an immutable ref to the bullet, since we don't need
    /// to change its value to draw it.
    fn render(&self, phi: &mut Phi);

    /// Again, immutable, since we don't need to change its value to return its Rectangle
    fn rect(&self) -> Rectangle;
}


// ##############################################################
// impls
// ##############################################################
impl Bullet for DivergentBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        self.total_time += dt;
        self.origin_y -= BULLET_SPEED_SLOW * dt;

        // If the bullet leaves the screen, delete it.
        let (w, h) = phi.output_size();
        let rect = self.rect();

        if rect.x > w || rect.x < 0.0 || rect.y > h || rect.y < 0.0 {
            None
        } else {
            Some(self)
        }

    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        let rendering = self.rect().to_sdl();
        match rendering {
            None => panic!("Unable to render DivergentBullet!"),
            Some(bullet) => phi.renderer.fill_rect(bullet),
        }
    }

    fn rect(&self) -> Rectangle {
        let time_delta = self.total_time / self.b;
        let dx = self.a * (time_delta.powi(3) - time_delta.powi(2));

        Rectangle {
            x: self.pos_x + dx,
            y: self.origin_y,
            w: BULLET_W,
            h: BULLET_H
        }
    }
}


impl Bullet for SineBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        self.total_time += dt;
        self.origin_y -= BULLET_SPEED * dt;

        let (w, _) = phi.output_size();
        if self.rect().x > w {
            None
        } else {
            Some(self)
        }

    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        let rendering = self.rect().to_sdl();
        match rendering {
            None => panic!("Couldn't render the SineBullet!"),
            Some(bullet) => phi.renderer.fill_rect(bullet),
        }
    }

    fn rect(&self) -> Rectangle {
        let dx = self.amplitude * f64::sin(self.angular_vel * self.total_time);
        Rectangle {
            x: self.pos_x + dx,
            y: self.origin_y,
            w: BULLET_W,
            h: BULLET_H,
        }
    }
}


impl RectBullet {
    fn new(x: f64, y: f64) -> RectBullet {
        RectBullet {
            rect: Rectangle {
                x: x,
                y: y,
                w: BULLET_W,
                h: BULLET_H,
            }
        }
    }
}

impl Bullet for RectBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        let (w, _) = phi.output_size();
        self.rect.y -= BULLET_SPEED * dt;

        // if the bullet has left the screen, delete it
        if self.rect.x > w {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        let rendering = self.rect.to_sdl();
        match rendering {
            None => panic!("Unable to render RectBullet!"),
            Some(bullet) => phi.renderer.fill_rect(bullet),
        }
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }
}


pub fn spawn_bullets(cannon: CannonType, cannon1_x: f64,
                     cannon2_x: f64, cannons_y: f64) -> Vec<Box<Bullet>> {
    match cannon {
        CannonType::RectBullet =>
            vec![
                Box::new(RectBullet {
                    rect: Rectangle {
                        x: cannon1_x,
                        y: cannons_y,
                        w: BULLET_W,
                        h: BULLET_H,
                    }
                }),
                // Box::new(RectBullet {
                //     rect: Rectangle {
                //         x: cannon2_x,
                //         y: cannons_y,
                //         w: BULLET_W,
                //         h: BULLET_H,
                //     }
                // }),
            ],
        CannonType::SineBullet { amplitude, angular_vel } =>
            vec![
                Box::new(SineBullet {
                    pos_x: cannon1_x,
                    origin_y: cannons_y,
                    amplitude: amplitude,
                    angular_vel: angular_vel,
                    total_time: 0.0,
                }),
                // Box::new(SineBullet {
                //     pos_x: cannon2_x,
                //     origin_y: cannons_y,
                //     amplitude: amplitude,
                //     angular_vel: angular_vel,
                //     total_time: 0.0,
                // })
            ],

        CannonType::DivergentBullet { a, b } =>
            vec![
                Box::new(DivergentBullet {
                    pos_x: cannon1_x,
                    origin_y: cannons_y,
                    a: -a,
                    b: b,
                    total_time: 0.0,
                }),
                // Box::new(DivergentBullet {
                //     pos_x: cannon2_x,
                //     origin_y: cannons_y,
                //     a: a,
                //     b: b,
                //     total_time: 0.0,
                // })
            ]
        }
}
