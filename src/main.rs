// use std::borrow::BorrowMut;
// use std::cell::RefCell;

use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{self, Color, DrawMode};
use ggez::mint::{Point2, Vector2};
use ggez::{Context, ContextBuilder, GameResult};

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("my_game", "me").build()?;

    ctx.gfx.set_window_title("My Game");
    ctx.gfx.set_mode(WindowMode {
        width: 1600.0,
        height: 900.0,
        resizable: true,
        ..Default::default()
    })?;

    let my_game = MyGame::new(&mut ctx);

    event::run(ctx, event_loop, my_game);
}

struct MyGame {
    balls: Vec<Ball>,
    active_ball: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
struct Ball {
    point: Point2<f32>,
    velocity: Vector2<f32>,
    radius: f32,
    color: Color,
}

impl Ball {
    pub fn new(x: f32, y: f32, radius: f32, color: Color) -> Self {
        Self {
            point: Point2 { x, y },
            velocity: Vector2 { x: 0.0, y: 0.0 },
            radius,
            color,
        }
    }
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        MyGame {
            balls: vec![
                Ball::new(300.0, 300.0, 30.0, Color::RED),
                Ball::new(500.0, 500.0, 50.0, Color::GREEN),
            ],
            active_ball: None,
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mouse = &ctx.mouse;
        let cursor = mouse.position();

        if mouse.button_just_pressed(MouseButton::Left) {
            self.active_ball = None;

            for (i, ball) in self.balls.iter().enumerate() {
                if (ball.point.x - cursor.x).abs() < ball.radius
                    && (ball.point.y - cursor.y).abs() < ball.radius
                {
                    self.active_ball = Some(i);
                    break;
                }
            }
        } else if mouse.button_just_released(MouseButton::Left) {
            if let Some(active) = self.active_ball {
                let ball = &mut self.balls[active];

                let vx = ball.point.x - cursor.x;
                let vy = ball.point.y - cursor.y;

                ball.velocity.x = vx;
                ball.velocity.y = vy;
            }
        } else if !mouse.button_pressed(MouseButton::Left) {
            self.active_ball = None;
        }

        for ball in &mut self.balls {
            const SPEED: f32 = 0.5;
            const DECELERATION: f32 = 5.0;
            const BOUNCE_DECELERATION: f32 = 50.0;
            const MAX_VELOCITY: f32 = 200.0;

            clamp(&mut ball.velocity.x, -MAX_VELOCITY, MAX_VELOCITY);
            clamp(&mut ball.velocity.y, -MAX_VELOCITY, MAX_VELOCITY);

            ball.point.x += ball.velocity.x * SPEED;
            ball.point.y += ball.velocity.y * SPEED;

            slow(&mut ball.velocity.x, DECELERATION);
            slow(&mut ball.velocity.y, DECELERATION);

            let (width, height) = ctx.gfx.drawable_size();

            bounce(
                &mut ball.point.x,
                &mut ball.velocity.x,
                ball.radius,
                width - ball.radius,
                BOUNCE_DECELERATION,
            );
            bounce(
                &mut ball.point.y,
                &mut ball.velocity.y,
                ball.radius,
                height - ball.radius,
                BOUNCE_DECELERATION,
            );
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        for ball in &self.balls {
            let circle = graphics::Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                ball.point,
                ball.radius,
                0.1,
                ball.color,
            )?;

            canvas.draw(&circle, graphics::DrawParam::default());
        }

        if let Some(active) = self.active_ball {
            let ball = self.balls[active];

            let circle = graphics::Mesh::new_circle(
                ctx,
                DrawMode::stroke(10.0),
                ball.point,
                ball.radius,
                0.1,
                Color::WHITE,
            )?;

            canvas.draw(&circle, graphics::DrawParam::default());

            let line = graphics::Mesh::new_line(
                ctx,
                &[ball.point, ctx.mouse.position()],
                5.0,
                Color::WHITE,
            )?;

            canvas.draw(&line, graphics::DrawParam::default());
        }

        canvas.finish(ctx)
    }
}

fn clamp(value: &mut f32, min: f32, max: f32) {
    if *value > max {
        *value = max;
    } else if *value < min {
        *value = min;
    }
}

fn slow(vel: &mut f32, deceleration: f32) {
    if vel.abs() < deceleration {
        *vel = 0.0;
    } else {
        *vel -= deceleration * vel.signum();
    }
}

fn bounce(pos: &mut f32, vel: &mut f32, min: f32, max: f32, deceleration: f32) {
    if *pos <= min {
        *pos = min;
        slow(vel, deceleration);
        *vel *= -1.0;
    }
    if *pos >= max {
        *pos = max;
        slow(vel, deceleration);
        *vel *= -1.0;
    }
}
