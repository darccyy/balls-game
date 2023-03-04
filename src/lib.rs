use ggez::{mint::Point2, graphics::Color};

/// Vector of direction and magnitude
#[derive(Debug, Clone, Copy)]
pub struct AngleVec {
    /// Angle from +x axis
    pub direction: f32,
    /// Magnitude of vector
    pub magnitude: f32,
}

impl AngleVec {
    /// Convert xy values into angle vector
    pub fn from_xy(x: f32, y: f32) -> Self {
        let magnitude = (x * x + y * y).sqrt();
        let direction = y.atan2(x);

        Self {
            magnitude,
            direction,
        }
    }

    /// Convert angle vector into xy values
    pub fn to_xy(self) -> (f32, f32) {
        let x = self.magnitude * self.direction.cos();
        let y = self.magnitude * self.direction.sin();

        (x, y)
    }
}

/// Keep value between a min and max
pub fn clamp(value: &mut f32, min: f32, max: f32) {
    if *value > max {
        *value = max;
    } else if *value < min {
        *value = min;
    }
}

/// Apply deceleration to a value
///
/// Set value to zero if deceleration amount is more than value
pub fn slow(value: &mut f32, deceleration: f32) {
    if value.abs() < deceleration {
        *value = 0.0;
    } else {
        *value -= deceleration * value.signum();
    }
}

/// Ball with position and velocity
#[derive(Debug, Clone, Copy)]
pub struct Ball {
    /// Position of ball
    pub point: Point2<f32>,
    /// Velocity, as an angle vector
    pub velocity: AngleVec,
    /// Radius of ball
    pub radius: f32,
    /// Color of ball
    pub color: Color,
}

impl Ball {
    /// Acceleration magnitude
    pub const ACCELERATION: f32 = 0.3;
    /// Maximum absolute velocity
    pub const MAX_VELOCITY: f32 = 120.0;
    /// Deceleration amount for friction
    pub const DECELERATION: f32 = 2.0;
    /// Deceleration amount for bounce force
    pub const BOUNCE_DECELERATION: f32 = 2.0;

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
