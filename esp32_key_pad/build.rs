//! 构建脚本: esp-hal 标准模板。

fn main() {
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    println!("cargo:rerun-if-env-changed=MCU");
}
