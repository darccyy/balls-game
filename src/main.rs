use std::f32::consts::PI;

use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{self, Color, DrawMode};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::{Context, ContextBuilder, GameResult};

use balls_game::{clamp, slow, AngleVec};

fn main() -> GameResult {
    // Create game context
    let (mut ctx, event_loop) = ContextBuilder::new("balls_game", "darcy").build()?;

    // Change window properties
    ctx.gfx.set_window_title("Balls Game");
    ctx.gfx.set_mode(WindowMode {
        width: 1600.0,
        height: 900.0,
        resizable: true,
        ..Default::default()
    })?;

    // Create game state
    let game = BallsGame::new(&mut ctx);

    // Run game loop
    event::run(ctx, event_loop, game);
}

struct BallsGame {
    /// All balls in game
    balls: Vec<Ball>,
    /// Active ball, which is selected
    ///
    /// This should be a reference for safety, but that is difficult
    active_ball: Option<usize>,
}

/// Ball with position and velocity
#[derive(Debug, Clone, Copy)]
struct Ball {
    /// Position of ball
    point: Point2<f32>,
    /// Velocity, as an angle vector
    velocity: AngleVec,
    /// Radius of ball
    radius: f32,
    /// Color of ball
    color: Color,
}

impl Ball {
    /// Acceleration magnitude
    const ACCELERATION: f32 = 0.3;
    /// Maximum absolute velocity
    const MAX_VELOCITY: f32 = 120.0;
    /// Deceleration amount for friction
    const DECELERATION: f32 = 2.0;
    /// Deceleration amount for bounce force
    const BOUNCE_DECELERATION: f32 = 2.0;

    /// New ball with x, y, radius, color, and zero velocity
    pub fn new(x: f32, y: f32, radius: f32, color: Color) -> Self {
        Self {
            point: Point2 { x, y },
            velocity: AngleVec {
                direction: 0.0,
                magnitude: 0.0,
            },
            radius,
            color,
        }
    }
}

impl BallsGame {
    pub fn new(_ctx: &mut Context) -> BallsGame {
        BallsGame {
            balls: vec![
                Ball::new(300.0, 300.0, 30.0, Color::RED),
                Ball::new(500.0, 500.0, 50.0, Color::GREEN),
            ],
            active_ball: None,
        }
    }
}

impl EventHandler for BallsGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Reset game with 'R' key
        if ctx.keyboard.is_key_just_pressed(KeyCode::R) {
            *self = Self::new(ctx);
            return Ok(());
        }

        // Get mouse and cursor
        let mouse = &ctx.mouse;
        let cursor = mouse.position();

        // Move balls with mouse
        if mouse.button_just_pressed(MouseButton::Left) {
            // Change active ball
            // Default to None
            self.active_ball = None;

            for (i, ball) in self.balls.iter().enumerate() {
                // Mouse collides with ball
                if (ball.point.x - cursor.x).abs() < ball.radius
                    && (ball.point.y - cursor.y).abs() < ball.radius
                {
                    // Use this ball, and break loop
                    self.active_ball = Some(i);
                    break;
                }
            }
        } else if mouse.button_just_released(MouseButton::Left) {
            // Apply velocity to active ball, if exists
            if let Some(active) = self.active_ball {
                let ball = &mut self.balls[active];

                // Get velocity vector
                ball.velocity = AngleVec::from_xy(ball.point.x - cursor.x, ball.point.y - cursor.y);
                // Apply acceleration speed
                ball.velocity.magnitude *= Ball::ACCELERATION;
            }

            // Reset active ball
            self.active_ball = None;
        }

        // Loop balls
        for ball in &mut self.balls {
            // Apply min and max velocity
            clamp(
                &mut ball.velocity.magnitude,
                -Ball::MAX_VELOCITY,
                Ball::MAX_VELOCITY,
            );

            // Apply velocity to ball position
            let (vx, vy) = ball.velocity.to_xy();
            ball.point.x += vx;
            ball.point.y += vy;

            // Decrease velocity slowly for friction
            slow(&mut ball.velocity.magnitude, Ball::DECELERATION);

            // Size of canvas
            let (width, height) = ctx.gfx.drawable_size();

            // If ball is out of bounds, flip velocity direction and decrease velocity for bounce force
            if ball.point.x < ball.radius
                || ball.point.x > width - ball.radius
                || ball.point.y < ball.radius
                || ball.point.y > height - ball.radius
            {
                ball.velocity.direction *= -1.0;
                slow(&mut ball.velocity.magnitude, Ball::BOUNCE_DECELERATION);
            }

            // Ball x position is out of bounds
            // Change direction by a half rotation
            if ball.point.x < ball.radius {
                ball.velocity.direction += PI;
                ball.point.x = ball.radius;
            } else if ball.point.x > width - ball.radius {
                ball.velocity.direction += PI;
                ball.point.x = width - ball.radius;
            }

            // Ball y position is out of bounds
            if ball.point.y < ball.radius {
                ball.point.y = ball.radius;
            } else if ball.point.y > height - ball.radius {
                ball.point.y = height - ball.radius;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        // Draw balls
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

        // Draw active ball, if exists
        if let Some(active) = self.active_ball {
            let ball = self.balls[active];

            // Stroke circle
            let circle = graphics::Mesh::new_circle(
                ctx,
                DrawMode::stroke(10.0),
                ball.point,
                ball.radius,
                0.1,
                Color::WHITE,
            )?;
            canvas.draw(&circle, graphics::DrawParam::default());

            // Line to cursor
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
