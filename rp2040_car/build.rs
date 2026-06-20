//! 构建脚本: 把 memory.x 提供给链接器, 并配置 RP2040 链接参数。
//! 官方 embassy-rp 标准模板。

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // 把 memory.x 放到输出目录, 让链接器总能找到。
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // memory.x 改动时重新构建。
    println!("cargo:rerun-if-changed=memory.x");

    // RP2040 链接配置 (官方 embassy-rp 模板)
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
