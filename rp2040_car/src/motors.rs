//! L298N ×2 四驱电机驱动 (差速控制)
//!
//! 把左右两侧各当作一个整体通道: 每侧一路 PWM(调速) + 两个方向脚(正/反)。
//! 同侧的前后两个电机在硬件上并联到同一组 PWM+DIR。
//!
//! 接线对应 config.rs:
//!   左侧 PWM = GP8 (slice4 chA), DIR1=FWD=GP12, DIR2=REV=GP13
//!   右侧 PWM = GP11(slice5 chA), DIR1=FWD=GP14, DIR2=REV=GP15
//!
//! 方向脚用具体 PIN 类型存储, 避免 AnyPin 擦除的 API 差异。

use embassy_rp::gpio::{Level, Output};
use embassy_rp::pwm::{Pwm, SetDutyCycle};

use crate::config;

#[derive(Clone, Copy)]
enum Dir {
    Stop,
    Forward,
    Reverse,
}

/// 四驱差速控制器。
/// embassy 0.10: Pwm<'d> 与 Output<'d> 均无外设泛型(已在构造时擦除)。
pub struct Motors {
    left_pwm: Pwm<'static>,
    right_pwm: Pwm<'static>,
    left_fwd: Output<'static>,
    left_rev: Output<'static>,
    right_fwd: Output<'static>,
    right_rev: Output<'static>,
}

impl Motors {
    pub fn new(
        left_pwm: Pwm<'static>,
        right_pwm: Pwm<'static>,
        left_fwd: Output<'static>,
        left_rev: Output<'static>,
        right_fwd: Output<'static>,
        right_rev: Output<'static>,
    ) -> Self {
        let mut s = Self {
            left_pwm, right_pwm, left_fwd, left_rev, right_fwd, right_rev,
        };
        s.stop();
        s
    }

    /// 立即停车: 占空比归零, 方向脚全低。
    pub fn stop(&mut self) {
        apply_dir(&mut self.left_fwd, &mut self.left_rev, Dir::Stop);
        apply_dir(&mut self.right_fwd, &mut self.right_rev, Dir::Stop);
        let _ = self.left_pwm.set_duty_cycle_fully_off();
        let _ = self.right_pwm.set_duty_cycle_fully_off();
    }

    /// 设置两侧速度。
    /// - `left`/`right`: -1000..1000, 正为前进, 负为后退, 0 为停。
    /// - `speed_limit`: 0..=100, 整体限速百分比。
    pub fn drive(&mut self, left: i16, right: i16, speed_limit: u8) {
        let limit_pct = (speed_limit as i32).clamp(0, 100);

        let (duty, dir) = scale(left, limit_pct);
        apply_dir(&mut self.left_fwd, &mut self.left_rev, dir);
        let _ = self.left_pwm.set_duty_cycle(duty);

        let (duty, dir) = scale(right, limit_pct);
        apply_dir(&mut self.right_fwd, &mut self.right_rev, dir);
        let _ = self.right_pwm.set_duty_cycle(duty);
    }
}

fn apply_dir(fwd: &mut Output<'static>, rev: &mut Output<'static>, dir: Dir) {
    match dir {
        Dir::Stop => { fwd.set_low(); rev.set_low(); }
        Dir::Forward => { fwd.set_high(); rev.set_low(); }
        Dir::Reverse => { fwd.set_low(); rev.set_high(); }
    }
}

/// 把 -1000..1000 的速度值, 配合限速百分比, 映射到 (PWM duty, 方向)。
fn scale(val: i16, limit_pct: i32) -> (u16, Dir) {
    let mag = (val.unsigned_abs() as i32).min(1000); // 0..1000
    let scaled = (mag * limit_pct) / 100; // 0..1000
    let duty = ((scaled as u32) * (config::PWM_TOP as u32) / 1000) as u16;
    let dir = if val > 0 { Dir::Forward } else if val < 0 { Dir::Reverse } else { Dir::Stop };
    (duty, dir)
}
