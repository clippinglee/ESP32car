#include <WiFi.h>
#include <Arduino.h>
#include <WebServer.h>
#include "CarControl/CarControl.hpp"
const char* ssid = "mycar";      // 热点名称
const char* password = "12345678";     // 密码（至少8位）

WebServer server(80);

const char MAIN_page[] = R"=====(
<!DOCTYPE html>
<html>
<head>
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
    <title>ESP32 小车</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
            background: #f8f9fa;
            height: 100vh;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            touch-action: manipulation;
            overflow: hidden;
        }
        .container {
            width: 95vmin;
            max-width: 500px;
            display: grid;
            grid-template-columns: 1fr 1fr 1fr;
            gap: 12px;
        }
        .btn {
            aspect-ratio: 1 / 1; /* 正方形 */
            font-size: 8vmin;
            border: none;
            border-radius: 20px;
            box-shadow: 0 6px 12px rgba(0,0,0,0.2);
            display: flex;
            justify-content: center;
            align-items: center;
            user-select: none;
            -webkit-tap-highlight-color: transparent;
            transition: transform 0.1s, background-color 0.2s;
        }
        .btn:active {
            transform: scale(0.95);
            box-shadow: 0 3px 6px rgba(0,0,0,0.2);
        }
        #forward { background: #4CAF50; color: white; }
        #backward { background: #f44336; color: white; }
        #left { background: #FFC107; color: black; }
        #right { background: #2196F3; color: white; }
        #stop { 
            background: #e74c3c; 
            color: white; 
            font-size: 10vmin;
        }
        #left-cycle { background: #FF9800; color: white; }
        #right-cycle { background: #9C27B0; color: white; }

        /* 单独设置后退按钮位置：跨三列居中 */
        #backward {
            grid-column: span 3;
        }
    </style>
</head>
<body>
    <div class="container">
        <!-- 第一行：左转圈 | 前进 | 右转圈 -->
        <button id="left-cycle" class="btn" ontouchstart="cmd('/left_cycle')" ontouchend="cmd('/stop')">↺</button>
        <button id="forward" class="btn" ontouchstart="cmd('/forward')" ontouchend="cmd('/stop')">↑</button>
        <button id="right-cycle" class="btn" ontouchstart="cmd('/right_cycle')" ontouchend="cmd('/stop')">↻</button>
        
        <!-- 第二行：左 | 停止 | 右 -->
        <button id="left" class="btn" ontouchstart="cmd('/left')" ontouchend="cmd('/stop')">←</button>
        <button id="stop" class="btn" onclick="cmd('/stop')">⏹️</button>
        <button id="right" class="btn" ontouchstart="cmd('/right')" ontouchend="cmd('/stop')">→</button>
        
        <!-- 第三行：后退（跨三列） -->
        <button id="backward" class="btn" ontouchstart="cmd('/backward')" ontouchend="cmd('/stop')">↓</button>
    </div>

    <script>
        function cmd(path) {
            fetch(path).catch(e => console.log('Cmd:', path, e));
        }
    </script>
</body>
</html>
)=====";

void handleRoot() {
    server.send(200, "text/html", MAIN_page);
}

smartcarContol* car = NULL;
void car_impl(smartcarContol* impl)
{
    car = impl;
}

void processCarMovement(int dirtion)
{
  int getDiretion = dirtion;
  uint8_t laststatus;
  car->comunicate_connected = 1;
  switch (getDiretion)
  {

    case FORWARD:
      car->laststatus = car->status;
      car->car_forward();
      Serial.println("car_forward");
      break;

    case BACKWARD:
      car->laststatus = car->status;
      car->car_reverse();
      Serial.println("car_reverse");
      break;

    case LEFT:
      car->laststatus = car->status;
      car->car_turnLeft();
      Serial.println("car_turnLeft");
      break;

    case RIGHT:
      car->laststatus = car->status;
      car->car_turnRight();      
      Serial.println("car_turnRight");
      break;

    case LEFT_CYCLE:
      car->laststatus = car->status;
      car->car_turnLeftCycle();
      Serial.println("car_turnLeftCycle");
      break;
    case RIGHT_CYCLE:
      car->car_turnRightCycle();
      Serial.println("car_turnRightCycle");
      break;

    case STOP:
      car->laststatus = car->status;
      car->car_stop();
      Serial.println("car_stop");
      break;
    default:

      break;
  }
}

void handleForward()   { processCarMovement(FORWARD); Serial.println("前进");   server.send(200, "text/plain", "OK"); }
void handleBackward()  { processCarMovement(BACKWARD); Serial.println("后退");   server.send(200, "text/plain", "OK"); }
void handleLeft()      { processCarMovement(LEFT); Serial.println("左转");   server.send(200, "text/plain", "OK"); }
void handleRight()     { processCarMovement(RIGHT); Serial.println("右转");   server.send(200, "text/plain", "OK"); }
void handleStop()      { processCarMovement(STOP); Serial.println("停止");   server.send(200, "text/plain", "OK"); }
void handleLeftCycle() {  processCarMovement(LEFT_CYCLE);  Serial.println("左转圈");server.send(200, "text/plain", "OK");}
void handleRightCycle() {  processCarMovement(RIGHT_CYCLE);  Serial.println("右转圈");server.send(200, "text/plain", "OK");}
// ⭐️ 关键：捕获所有其他请求并重定向到主页（实现自动弹窗）
void handleCaptivePortal() {
   String url = server.uri();
    Serial.print("Captive Portal 请求: ");
    Serial.println(url);

    if (url == "/generate_204") {
        // Android 探测：必须返回 204 且无 body
        server.send(204, "text/plain", "");
        return;
    }

    // iOS / macOS
    if (url == "/hotspot-detect.html") {
        server.send(200, "text/html", "<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>");
        return;
    }

    // Windows
    if (url == "/connecttest.txt" || url == "/fwlink") {
        server.send(200, "text/plain", "Microsoft Connect Test");
        return;
    }

    // 其他未知请求：重定向到主页
    server.sendHeader("Location", "/", true);
    server.send(302, "text/plain", "");
}
void handleFavicon() {
    server.send(204); // HTTP 204 = No Content（标准做法）
}

void wifi_main(void *pvParameters);
void wifi_init() {

    // 启动 AP 模式
    WiFi.softAP(ssid, password);
    IPAddress IP = WiFi.softAPIP();
    Serial.print("AP IP: ");
    Serial.println(IP);

      // 注册路由
    server.on("/", HTTP_GET, handleRoot);
    server.on("/forward", HTTP_GET, handleForward);
    server.on("/backward", HTTP_GET, handleBackward);
    server.on("/left", HTTP_GET, handleLeft);
    server.on("/right", HTTP_GET, handleRight);
    server.on("/stop", HTTP_GET, handleStop);
    server.on("/left_cycle", HTTP_GET, handleLeftCycle);
    server.on("/right_cycle", HTTP_GET, handleRightCycle);

    // Captive Portal 探测路径
    server.on("/generate_204", HTTP_GET, handleCaptivePortal);
    server.on("/hotspot-detect.html", HTTP_GET, handleCaptivePortal);
    server.on("/connecttest.txt", HTTP_GET, handleCaptivePortal);
    server.on("/fwlink", HTTP_GET, handleCaptivePortal);
    server.on("/favicon.ico", HTTP_GET, handleFavicon);

    server.onNotFound(handleCaptivePortal); // 兜底

    server.begin();
    Serial.println("Captive Portal 已启用，连接后将自动弹出控制页面");
    xTaskCreatePinnedToCore(wifi_main, "wifi_mian", 100000, NULL, 5, NULL, 0);
}

void wifi_main(void *pvParameters) {
    Serial.println("-----------------wifi_main----------------------------------");
    while(1)
    {
        server.handleClient();
        delay(1);
    }
 
}