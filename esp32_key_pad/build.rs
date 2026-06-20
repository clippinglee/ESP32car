//! 构建脚本: esp-hal 标准模板。

fn main() {
    // esp-hal 需要链接时优化
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    println!("cargo:rustc-link-arg-bins=-Tromfix.x");

    // 让 esp-hal 的 build.rs 能找到正确的 chip 配置
    println!("cargo:rerun-if-env-changed=MCU");
}
