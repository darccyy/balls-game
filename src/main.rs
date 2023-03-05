use std::f32::consts::PI;
use std::ops::RangeInclusive;

use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{self, Color, DrawMode};
use ggez::input::keyboard::KeyCode;
use ggez::mint::Point2;
use ggez::{Context, ContextBuilder, GameResult};

use balls_game::{clamp, slow, AngleVec, Ball, Collides};
use rand::Rng;

const BALL_COUNT: RangeInclusive<u32> = 5..=40;

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

impl BallsGame {
    /// Reset game
    pub fn new(ctx: &mut Context) -> BallsGame {
        // let mut balls = vec![
        //     Ball::new(300.0, 300.0, 30.0, Color::RED),
        //     Ball::new(500.0, 500.0, 50.0, Color::GREEN),
        //     Ball::new(700.0, 200.0, 80.0, Color::BLUE),
        //     Ball::new(650.0, 600.0, 10.0, Color::YELLOW),
        // ];

        let mut balls = vec![];

        // Create random balls
        let (width, height) = ctx.gfx.drawable_size();
        let mut rng = rand::thread_rng();
        for _ in 0..rng.gen_range(BALL_COUNT) {
            let radius = rng.gen_range(30.0..90.0);

            balls.push(Ball::new(
                rng.gen_range(radius..(width - radius)),
                rng.gen_range(radius..(height - radius)),
                radius,
                Color::from_rgb(
                    rng.gen_range(50..=255),
                    rng.gen_range(50..=255),
                    rng.gen_range(50..=255),
                ),
            ));
        }

        // Sort so smallest balls are last - Drawn in front
        balls.sort_by(|a, b| b.radius.partial_cmp(&a.radius).unwrap());

        BallsGame {
            balls,
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

        // Move active ball to cursor if 'Z' key is pressed
        if ctx.keyboard.is_key_pressed(KeyCode::Z) {
            if let Some(active) = self.active_ball {
                let ball = &mut self.balls[active];
                ball.point.x = cursor.x;
                ball.point.y = cursor.y;
            }
        }

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

            // Use slow mode
            let speed = if ctx.keyboard.is_key_pressed(KeyCode::C) {
                Ball::SLOW_MAGNITUDE
            } else {
                1.0
            };

            // Apply velocity to ball position
            let mut velocity = ball.velocity;
            velocity.magnitude *= speed;
            let (vx, vy) = velocity.to_xy();
            ball.point.x += vx;
            ball.point.y += vy;

            // Decrease velocity slowly for friction
            slow(&mut ball.velocity.magnitude, Ball::DECELERATION * speed);

            // Size of canvas
            let (width, height) = ctx.gfx.drawable_size();

            // If ball is out of bounds, flip velocity direction and decrease velocity for bounce force
            if ball.point.x < ball.radius
                || ball.point.x > width - ball.radius
                || ball.point.y < ball.radius
                || ball.point.y > height - ball.radius
            {
                ball.velocity.direction *= -1.0;
                slow(
                    &mut ball.velocity.magnitude,
                    Ball::BOUNCE_DECELERATION * speed,
                );
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

        let mut new_balls = vec![];

        // Check for collisions with other balls
        for (i, ball) in self.balls.iter().enumerate() {
            let mut ball = ball.clone();
            // Default to not colliding
            ball.is_colliding = false;

            for (j, other) in self.balls.iter().enumerate() {
                // Ignore collision with self
                if i == j {
                    continue;
                }

                // Check collision
                if ball.collides(other) {
                    ball.is_colliding = true;

                    let x = ball.point.x - other.point.x;
                    let y = ball.point.y - other.point.y;

                    // let sum_magnitude = ball.velocity.magnitude + other.velocity.magnitude;

                    let radius_ratio = ball.radius / other.radius;

                    let new = AngleVec {
                        direction: y.atan2(x),
                        // magnitude: sum_magnitude / 2.0 / radius_ratio,
                        magnitude: 10.0 / radius_ratio,
                    };

                    ball.velocity = AngleVec {
                        direction: (new.direction + ball.velocity.direction) / 2.0,
                        magnitude: (new.magnitude + ball.velocity.magnitude) / 2.0,
                    }
                }
            }

            new_balls.push(ball);
        }

        self.balls = new_balls;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        // Draw balls
        for ball in &self.balls {
            // Fill circle
            let circle = graphics::Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                ball.point,
                ball.radius,
                0.1,
                ball.color,
            )?;
            canvas.draw(&circle, graphics::DrawParam::default());

            // Draw a cross if ball is colliding
            if ball.is_colliding && ctx.keyboard.is_key_pressed(KeyCode::M) {
                let Point2 { x, y } = ball.point;
                let radius = ball.radius;

                canvas.draw(
                    &graphics::Mesh::new_line(
                        ctx,
                        &[
                            Point2 {
                                x: x - radius,
                                y: y - radius,
                            },
                            Point2 {
                                x: x + radius,
                                y: y + radius,
                            },
                        ],
                        10.0,
                        Color::BLACK,
                    )?,
                    graphics::DrawParam::default(),
                );
                canvas.draw(
                    &graphics::Mesh::new_line(
                        ctx,
                        &[
                            Point2 {
                                x: x + radius,
                                y: y - radius,
                            },
                            Point2 {
                                x: x - radius,
                                y: y + radius,
                            },
                        ],
                        10.0,
                        Color::BLACK,
                    )?,
                    graphics::DrawParam::default(),
                );
            }

            // Line velocity
            if ctx.keyboard.is_key_pressed(KeyCode::M) {
                let (vx, vy) = AngleVec {
                    magnitude: ball.velocity.magnitude * 3.0,
                    ..ball.velocity
                }
                .to_xy();
                let velocity_point = Point2 {
                    x: vx + ball.point.x,
                    y: vy + ball.point.y,
                };

                let line = graphics::Mesh::new_line(
                    ctx,
                    &[ball.point, velocity_point],
                    10.0,
                    Color::WHITE,
                )?;
                canvas.draw(&line, graphics::DrawParam::default());
            }
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
            let cursor = ctx.mouse.position();
            let velocity_point = Point2 {
                x: ball.point.x * 2.0 - cursor.x,
                y: ball.point.y * 2.0 - cursor.y,
            };

            let line = graphics::Mesh::new_line(ctx, &[cursor, velocity_point], 5.0, Color::WHITE)?;
            canvas.draw(&line, graphics::DrawParam::default());
        }

        canvas.finish(ctx)
    }
}
