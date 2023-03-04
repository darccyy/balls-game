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
