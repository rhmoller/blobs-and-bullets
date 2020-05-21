pub struct TwinStick {
    pub move_x_axis: f64,
    pub move_y_axis: f64,
    pub aim_x_axis: f64,
    pub aim_y_axis: f64,
    pub shoot: bool,
}

impl TwinStick {
    pub fn new() -> Self {
        TwinStick {
            move_x_axis: 0.,
            move_y_axis: 0.,
            aim_x_axis: 0.,
            aim_y_axis: 0.,
            shoot: false,
        }
    }
}
