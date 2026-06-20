//! 差速运动学: 摇杆 (x, y) -> 左右轮速度
//!
//! 坐标约定:
//!   axis_y > 0 = 前进, axis_y < 0 = 后退
//!   axis_x > 0 = 向右, axis_x < 0 = 向左
//! 范围均为 -1000..1000。
//!
//! 差速公式:
//!   left  = clamp(y + x)   // 右转时左轮加速、右轮减速
//!   right = clamp(y - x)
//!
//! 这样摇杆居中 -> 两轮停; 推满向前 -> 两轮全速前进;
//! 摇杆居中右推 -> 左轮正、右轮负, 原地右转。

/// 返回 (左轮速度, 右轮速度), 范围 -1000..1000。
pub fn arcade_mix(axis_x: i16, axis_y: i16) -> (i16, i16) {
    // i32 防止溢出
    let lx = axis_x as i32;
    let ly = axis_y as i32;
    let left = (ly + lx).clamp(-1000, 1000) as i16;
    let right = (ly - lx).clamp(-1000, 1000) as i16;
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopped() {
        assert_eq!(arcade_mix(0, 0), (0, 0));
    }

    #[test]
    fn full_forward() {
        assert_eq!(arcade_mix(0, 1000), (1000, 1000));
    }

    #[test]
    fn spin_right() {
        // 居中右推: 左正右负 = 原地右转
        assert_eq!(arcade_mix(1000, 0), (1000, -1000));
    }

    #[test]
    fn spin_left() {
        assert_eq!(arcade_mix(-1000, 0), (-1000, 1000));
    }
}
