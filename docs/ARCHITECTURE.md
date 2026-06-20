# 智能小车系统架构文档

## 目录

1. [系统概述](#1-系统概述)
2. [系统框图](#2-系统框图)
3. [硬件引脚映射](#3-硬件引脚映射)
4. [通信协议](#4-通信协议)
5. [软件架构](#5-软件架构)
6. [编译与烧录](#6-编译与烧录)
7. [安全机制](#7-安全机制)

---

## 1. 系统概述

本项目是一个基于 Rust `no_std` 的无线遥控智能小车系统,采用双 MCU 架构:

- **ESP32 (遥控端)**: 读取摇杆/按钮输入,通过 nRF24L01 以 100Hz 发送控制包
- **RP2040 (车控端)**: 接收控制包,驱动四轮差速电机,回传遥测数据
- **nRF24L01**: 2.4GHz 无线模块,支持自动应答(ACK)和动态载荷
- **protocol 共享协议库**: `no_std` Rust crate,两端共用,确保协议一致性

通信链路为单向控制(遥控器→小车) + 反向遥测(小车→遥控器 ACK payload),无需双向同时通信。

### 技术栈

| 组件 | 技术选型 |
|------|---------|
| 语言 | Rust (no_std) |
| 异步运行时 | embassy (ESP32: esp-hal-embassy, RP2040: embassy-rp) |
| 序列化 | postcard (no_std 兼容) |
| 无线驱动 | rf24-rs (embedded-hal 1.0) |
| 无线模块 | nRF24L01+ (2.4GHz, 动态载荷, ACK payload) |
| 电机驱动 | L298N x2 (双路 H 桥) |
| 调试日志 | defmt (RP2040), esp-println (ESP32) |

---

## 2. 系统框图

### 2.1 硬件连接总览

```
┌─────────────────────────────────────────────────────────────┐
│                     遥控器 (ESP32)                          │
│                                                             │
│  ┌──────────┐    ┌──────────┐    ┌───────────────────────┐ │
│  │  摇杆     │    │  按钮 x4  │    │  nRF24L01 (PTX)      │ │
│  │  X:GPIO36 │    │  A:GPIO0  │    │  SPI2 (VSPI)        │ │
│  │  Y:GPIO39 │    │  B:GPIO17 │    │  SCK=18, MOSI=23    │ │
│  │  (ADC1)   │    │  C:GPIO25 │    │  MISO=19, CSN=5     │ │
│  │           │    │  D:GPIO26 │    │  CE=4                │ │
│  └─────┬─────┘    └─────┬─────┘    └──────────┬──────────┘ │
│        │                │                     │             │
│        └────────────────┴─────────────────────┘             │
│                         │                                   │
│                    ┌────┴────┐                              │
│                    │ ESP32   │                              │
│                    │  MCU    │                              │
│                    └────┬────┘                              │
│                         │                                   │
│                    LED: GPIO2                               │
└─────────────────────────┼───────────────────────────────────┘
                          │
                    2.4GHz 无线
                   (CH76, 2.476GHz)
                          │
┌─────────────────────────┼───────────────────────────────────┐
│                    ┌────┴────┐                              │
│                    │ RP2040  │                              │
│                    │  MCU    │                              │
│                    └────┬────┘                              │
│                         │                                   │
│  ┌──────────────────────┴──────────────────────────────┐    │
│  │              nRF24L01 (PRX)                          │    │
│  │  SPI0: SCK=GP2, MOSI=GP3, MISO=GP4                  │    │
│  │       CSN=GP5, CE=GP6                                │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              L298N 电机驱动 x2                        │    │
│  │                                                      │    │
│  │  ┌────────────────┐    ┌────────────────┐           │    │
│  │  │  左侧电机       │    │  右侧电机       │           │    │
│  │  │  PWM: GP8      │    │  PWM: GP10     │           │    │
│  │  │  FWD: GP12     │    │  FWD: GP14     │           │    │
│  │  │  REV: GP13     │    │  REV: GP15     │           │    │
│  │  └────────┬───────┘    └────────┬───────┘           │    │
│  │           │                     │                    │    │
│  │  ┌────────┴───────┐    ┌───────┴────────┐           │    │
│  │  │ 左前轮 + 左后轮  │    │ 右前轮 + 右后轮  │           │    │
│  │  │ (并联)          │    │ (并联)          │           │    │
│  │  └────────────────┘    └────────────────┘           │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  LED: GP25 (链路状态指示)                                    │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 数据流

```
┌─────────┐    ADC采样     ┌──────────┐   postcard   ┌──────────┐
│  摇杆    │ ────────────> │  ESP32   │ ──────────> │ nRF24L01 │
│  (X/Y)  │               │  MCU     │   nRF发送    │  (PTX)   │
└─────────┘               └──────────┘              └────┬─────┘
                                                          │
                                                    2.4GHz 无线
                                                          │
┌─────────┐   postcard     ┌──────────┐   SPI读取   ┌────┴─────┐
│  电机    │ <──────────── │  RP2040  │ <───────── │ nRF24L01 │
│  (L298N) │   PWM+DIR     │  MCU     │   nRF接收   │  (PRX)   │
└─────────┘               └──────────┘              └──────────┘
                                                          │
                                                    ACK payload
                                                    (遥测包回传)
```

---

## 3. 硬件引脚映射

### 3.1 ESP32 遥控器

| 功能 | GPIO | 说明 |
|------|------|------|
| **nRF24L01 SPI (VSPI/SPI2)** | | |
| SCK | 18 | SPI 时钟 |
| MOSI | 23 | SPI 主出从入 |
| MISO | 19 | SPI 主入从出 |
| CSN | 5 | SPI 片选 (软件控制) |
| CE | 4 | nRF24 收发使能 |
| IRQ | 16 | nRF24 中断 (可选,当前未用) |
| **摇杆 (ADC1)** | | |
| X 轴 | 36 | ADC1_CH0 (仅输入,RTC 引脚) |
| Y 轴 | 39 | ADC1_CH3 (仅输入,RTC 引脚) |
| **按钮 (低电平有效)** | | |
| BTN_A | 0 | 板载 Boot 按钮,内上拉 |
| BTN_B | 17 | 外接按钮,内上拉 |
| BTN_C | 25 | 外接按钮,内上拉 |
| BTN_D | 26 | 外接按钮,内上拉 |
| **指示灯** | | |
| LED | 2 | 板载 LED,发送成功时亮 |

**ADC 参数:**
- 分辨率: 12 bit (0..4095)
- 中值: 2047 (摇杆居中)
- 死区: ±100 (偏移小于此值视为零)

### 3.2 RP2040 车控

| 功能 | GPIO | 说明 |
|------|------|------|
| **nRF24L01 SPI (SPI0)** | | |
| SCK | GP2 | SPI 时钟 |
| MOSI | GP3 | SPI 主出从入 |
| MISO | GP4 | SPI 主入从出 |
| CSN | GP5 | SPI 片选 (软件控制) |
| CE | GP6 | nRF24 收发使能 |
| IRQ | GP7 | nRF24 中断 (可选,当前未用) |
| **L298N 左侧电机** | | |
| PWM | GP8 | slice4 channel A |
| FWD (正转) | GP12 | |
| REV (反转) | GP13 | |
| **L298N 右侧电机** | | |
| PWM | GP10 | slice5 channel A |
| FWD (正转) | GP14 | |
| REV (反转) | GP15 | |
| **指示灯** | | |
| LED | GP25 | 板载 LED,链路正常时亮 |

**PWM 参数:**
- 系统时钟: 125 MHz
- 分频系数: 125
- 计数上限 (TOP): 32767
- PWM 频率: 125MHz / 125 / 32767 ≈ 30.5 Hz

---

## 4. 通信协议

### 4.1 无线配置

| 参数 | 值 | 说明 |
|------|-----|------|
| 频率 | 2.476 GHz | CH76 (2400 + 76 MHz) |
| 配对地址 | `[0xE7, 0xE7, 0xE7, 0xE7, 0xE7]` | 5 字节,两端一致 |
| 传输速率 | 1 Mbps | nRF24L01 默认 |
| 载荷模式 | 动态载荷 | 最大 32 字节 |
| 自动应答 | 启用 | ACK payload 机制 |
| 重传次数 | 2 次 | 每次间隔 500μs |
| 更新频率 | 100 Hz | 每 10ms 发送一包 |
| 失控超时 | 300 ms | 超时未收到包则停车 |

### 4.2 ControlPacket (遥控器 → 车控)

```
字节偏移  字段           类型     说明
─────────────────────────────────────────────────
0         magic          u8      魔数,固定 0xA5
1-2       seq            u16     序列号,每包 +1 (小端)
3-4       axis_x         i16     摇杆 X 轴 (-1000..1000)
5-6       axis_y         i16     摇杆 Y 轴 (-1000..1000)
7-8       buttons        u16     按键位图
9         speed_limit    u8      限速百分比 (0..=100)
10-11     crc            u16     CRC-16/CCITT-FALSE 校验
```

**总长度: 12 字节** (postcard 序列化后)

**按键位图定义:**

| bit | 按键 | 常量名 |
|-----|------|--------|
| 0 | BTN_A | `button::BTN_A` |
| 1 | BTN_B | `button::BTN_B` |
| 2 | BTN_C | `button::BTN_C` |
| 3 | BTN_D | `button::BTN_D` |
| 4 | BTN_SHIFT | `button::BTN_SHIFT` |

**特殊操作:**
- BTN_A + BTN_D 同时按住: 循环切换限速档位 (25% → 50% → 75% → 100% → 25%...)

### 4.3 TelemetryPacket (车控 → 遥控器, ACK payload)

```
字节偏移  字段           类型     说明
─────────────────────────────────────────────────
0-1       battery_mv     u16     电池电压 (mV), 0=未知
2-3       last_seq       u16     最后收到的控制包序列号
4         status         u8      状态位图
```

**状态位图:**

| bit | 含义 |
|-----|------|
| 0 | 链路正常 |
| 1 | 电机使能 |
| 2 | 低电压告警 |

### 4.4 CRC-16 校验

- 算法: CRC-16/CCITT-FALSE
- 多项式: 0x1021
- 初始值: 0xFFFF
- 校验范围: magic + seq + axis_x + axis_y + buttons + speed_limit (不含 crc 字段本身)

---

## 5. 软件架构

### 5.1 项目结构

```
smart-car/
├── Cargo.toml              # 工作区配置
├── protocol/               # 共享协议库 (no_std)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # 协议定义, CRC, 序列化
├── esp32_key_pad/          # ESP32 遥控器固件
│   ├── Cargo.toml
│   ├── build.rs
│   ├── .cargo/
│   │   └── config.toml
│   ├── rust-toolchain.toml
│   └── src/
│       ├── main.rs         # 入口: ADC采样 + nRF发送
│       ├── config.rs       # 引脚定义与硬件参数
│       └── radio.rs        # nRF24 遥测读取
├── rp2040_car/             # RP2040 车控固件
│   ├── Cargo.toml
│   ├── build.rs
│   ├── .cargo/
│   │   └── config.toml
│   ├── rust-toolchain.toml
│   └── src/
│       ├── main.rs         # 入口: nRF接收 + 电机驱动
│       ├── config.rs       # 引脚定义与硬件参数
│       ├── control.rs      # 差速运动学 (arcade mix)
│       ├── motors.rs       # L298N 四驱控制
│       └── radio.rs        # nRF24 数据解析
├── hardware/               # 硬件文档 (待补充)
└── docs/                   # 项目文档
    └── ARCHITECTURE.md     # 本文档
```

### 5.2 protocol 共享协议库

```
protocol/src/lib.rs
├── ControlPacket        # 遥控→车控控制包
│   ├── new()            # 构造 + 自动计算 CRC
│   └── verify()         # 验证 magic + CRC
├── TelemetryPacket      # 车控→遥控遥测包
├── button               # 按键位图常量
├── PAIR_ADDR            # 配对地址 [0xE7; 5]
├── CHANNEL              # 无线信道 76
├── TX_RATE_HZ           # 发送频率 100Hz
└── FAILSAFE_TIMEOUT_MS  # 失控超时 300ms
```

**设计原则:**
- `#![no_std]`, 不依赖标准库
- 使用 `postcard` 序列化 (比 `bincode` 更紧凑)
- 自动 CRC 计算,构造时零手动操作

### 5.3 ESP32 遥控器固件

```
esp32_key_pad/src/
├── main.rs              # embassy 异步入口
│   ├── ADC 初始化       # 摇杆 X/Y (ADC1)
│   ├── GPIO 初始化      # 按钮 x4 (内上拉, 低电平有效)
│   ├── SPI 初始化       # nRF24L01 (VSPI/SPI2, 8MHz)
│   ├── nRF24 初始化     # PTX 模式, 动态载荷, ACK
│   └── 主循环           # 100Hz: 采样→打包→发送→检查遥测
├── config.rs            # 所有 GPIO 编号集中定义
└── radio.rs             # try_read_ack() 读取遥测 ACK
```

**主循环流程:**

```
loop @ 100Hz:
  1. 读取摇杆 ADC (X, Y)
  2. 读取按钮状态
  3. 处理组合键 (A+D 切换限速)
  4. 构造 ControlPacket (自动 CRC)
  5. postcard 序列化
  6. nRF24 PTX 发送
  7. 检查 ACK payload (遥测包)
  8. 等待下一个 tick
```

### 5.4 RP2040 车控固件

```
rp2040_car/src/
├── main.rs              # embassy 异步入口
│   ├── PWM 初始化       # 左/右电机 PWM (GP8, GP10)
│   ├── GPIO 初始化      # 方向脚 (GP12-GP15)
│   ├── SPI 初始化       # nRF24L01 (SPI0, 8MHz, 阻塞)
│   ├── nRF24 初始化     # PRX 模式, 动态载荷, ACK
│   └── 主循环           # 100Hz: 接收→解析→驱动+失保护
├── config.rs            # 所有 GPIO 编号集中定义
├── control.rs           # arcade_mix() 差速运动学
├── motors.rs            # Motors 结构体, drive/stop
└── radio.rs             # parse_payload() + BlockingDelay
```

**主循环流程:**

```
loop @ 100Hz:
  1. 检查 nRF24 是否有数据
  2. 读取并反序列化 ControlPacket
  3. 验证 magic + CRC
  4. 更新 LAST_PKT_MS 时间戳
  5. 执行 arcade_mix (x,y → left,right)
  6. 调用 motors.drive(left, right, speed_limit)
  7. 亮 LED 表示链路正常
  8. 若超时 300ms 无有效包 → motors.stop() + 灭 LED
  9. 等待下一个 tick
```

### 5.5 差速运动学 (Arcade Mix)

```
摇杆坐标系:
  axis_y > 0 → 前进
  axis_y < 0 → 后退
  axis_x > 0 → 右转
  axis_x < 0 → 左转
  范围: -1000..1000

差速公式:
  left  = clamp(axis_y + axis_x, -1000, 1000)
  right = clamp(axis_y - axis_x, -1000, 1000)

典型场景:
  摇杆居中 (0, 0)       → 左轮 0,    右轮 0    (停车)
  推满向前 (0, 1000)    → 左轮 1000, 右轮 1000 (全速前进)
  拉满向后 (0, -1000)   → 左轮-1000, 右轮-1000 (全速后退)
  居中右推 (1000, 0)    → 左轮 1000, 右轮-1000 (原地右转)
  居中左推 (-1000, 0)   → 左轮-1000, 右轮 1000 (原地左转)
  前进右转 (500, 800)   → 左轮 1300→1000, 右轮 300 (弧线右转)
```

### 5.6 速度缩放

速度值 (-1000..1000) 经过限速百分比缩放后映射到 PWM 占空比:

```
scaled = |val| × speed_limit / 100    // 0..1000
duty   = scaled × PWM_TOP / 1000       // 0..32767
dir    = (val > 0) ? Forward
         (val < 0) ? Reverse
         : Stop
```

---

## 6. 编译与烧录

### 6.1 前置依赖

```bash
# 安装 Rust 工具链
# ESP32: 使用 esp 工具链 (含 Xtensa 支持)
# RP2040: 使用 stable 工具链 (ARM Cortex-M0)

# 安装烧录工具
cargo install espflash      # ESP32 烧录
cargo install probe-rs-cli  # RP2040 烧录 (J-Link/ST-Link)
cargo install elf2uf2-rs    # RP2040 UF2 烧录 (BOOTSEL 模式)
```

### 6.2 编译 protocol 库

```bash
# protocol 是共享库, 无需单独编译
# 编译固件时会自动作为依赖编译
```

### 6.3 编译 ESP32 遥控器固件

```bash
cd esp32_key_pad

# 编译 (Release)
cargo build --release

# 编译 + 烧录 + 监控
cargo run --release

# 仅烧录 (不监控)
espflash flash --monitor target/xtensa-esp32-none-elf/release/esp32_key_pad
```

**目标平台:** `xtensa-esp32-none-elf`
**工具链:** `esp` channel

### 6.4 编译 RP2040 车控固件

```bash
cd rp2040_car

# 编译 (Release)
cargo build --release

# 编译 + 烧录 + 监控 (J-Link/ST-Link)
cargo run --release

# 仅烧录 (UF2 模式: 按住 BOOTSEL 插入 USB)
elf2uf2-rs -d target/thumbv6m-none-eabi/release/rp2040_car

# 或使用 probe-rs
probe-rs run --chip RP2040 target/thumbv6m-none-eabi/release/rp2040_car
```

**目标平台:** `thumbv6m-none-eabi`
**工具链:** `stable` channel

### 6.5 工作区编译

```bash
# 编译整个工作区 (protocol + rp2040_car)
# 注意: esp32_key_pad 因工具链不同, 在 workspace 中被 exclude
cargo build --release -p rp2040_car
cargo build --release -p protocol
```

---

## 7. 安全机制

### 7.1 失控保护 (Failsafe)

当车控在 **300ms** 内未收到任何有效控制包时:

- 立即停止所有电机 (`motors.stop()`)
- LED 熄灭,指示链路断开
- 恢复通信后自动恢复控制

**触发条件:**
- 遥控器关闭或超出通信范围
- 无线干扰导致连续丢包
- nRF24L01 硬件故障

### 7.2 数据完整性校验

每个控制包包含 CRC-16/CCITT-FALSE 校验:

```
发送端 (ESP32):
  1. 构造 ControlPacket (crc 字段 = 0)
  2. 计算 CRC = crc16(magic + seq + axes + buttons + speed_limit)
  3. 写入 crc 字段
  4. 序列化并发送

接收端 (RP2040):
  1. 反序列化
  2. 验证 magic == 0xA5
  3. 重新计算 CRC, 与包中 crc 字段对比
  4. 校验失败 → 丢弃该包, 不更新电机状态
```

### 7.3 限速机制

遥控器可配置全局限速百分比 (25%/50%/75%/100%):

- **默认限速:** 50%
- **切换方式:** 同时按住 BTN_A + BTN_D
- **生效方式:** 限速值随控制包发送, 车端实时应用
- **作用:** 缩放所有电机速度, 保持差速比例不变

### 7.4 序列号检测

控制包包含 16 位递增序列号 (`seq`):

- 遥测包中的 `last_seq` 字段反馈车端最后收到的序列号
- 可用于检测丢包率和通信质量
- 车端仅使用最新有效包, 不做重放

### 7.5 摇杆死区

ADC 采样值经过死区处理:

- **死区范围:** ±100 (约 ADC 满量程的 2.4%)
- **效果:** 摇杆轻微偏移不会触发小车移动
- **映射:** 死区外的值线性缩放到 -1000..1000

### 7.6 LED 状态指示

**ESP32 遥控器:**
- LED 亮: 控制包发送成功
- LED 灭: 发送失败或未发送

**RP2040 车控:**
- LED 亮: 收到有效控制包 (链路正常)
- LED 灭: 超时停车或链路断开

---

## 附录

### A. 无线频率计算

```
频率 = 2400 + channel MHz
CH76 = 2400 + 76 = 2476 MHz = 2.476 GHz
```

### B. PWM 频率计算

```
PWM频率 = 系统时钟 / 分频系数 / (TOP + 1)
        = 125,000,000 / 125 / 32,768
        ≈ 30,517 Hz ≈ 30.5 Hz
```

### C. 协议依赖

```
protocol crate
├── serde (no_std, derive)
└── postcard (no_std)

esp32_key_pad
├── protocol
├── esp-hal 1.0 (ESP32)
├── esp-hal-embassy 0.7
├── embassy-executor 0.7
├── embassy-time 0.4
├── rf24-rs 0.3 (no_std)
└── postcard 1.1

rp2040_car
├── protocol
├── embassy-rp 0.10
├── embassy-executor 0.10
├── embassy-time 0.5
├── rf24-rs 0.3 (no_std)
├── postcard 1.1
├── embedded-hal 1.0
└── portable-atomic 1.0
```

### D. 已知限制

1. **单向控制:** 遥控器不能主动查询车端状态, 只能通过 ACK payload 获取遥测
2. **无加密:** 通信数据为明文, 任何人可在同频段监听
3. **单频道:** 所有使用相同地址/频道的设备会互相干扰
4. **电池监测:** TelemetryPacket 中 `battery_mv` 当前未实现 (车端 ADC 未接入)
5. **按键有限:** 仅 4 个功能按钮 + 1 个组合键操作
