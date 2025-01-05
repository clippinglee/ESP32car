
#include <Arduino.h>
#include <WiFi.h>
#include <AsyncTCP.h>
#include <ESPAsyncWebServer.h>
#include "CarControl/CarControl.hpp"


#define FORWARD 1
#define BACKWARD 2
#define LEFT 3
#define RIGHT 4
#define FORWARD_LEFT 5
#define FORWARD_RIGHT 6
#define BACKWARD_LEFT 7
#define BACKWARD_RIGHT 8

#define S_FORWARD 11
#define S_BACKWARD 12
#define S_LEFT 13
#define S_RIGHT 14
#define S_FORWARD_LEFT 15
#define S_FORWARD_RIGHT 16
#define S_BACKWARD_LEFT 17
#define S_BACKWARD_RIGHT 18

#define M_FORWARD 21
#define M_BACKWARD 22
#define M_LEFT 23
#define M_RIGHT 24
#define M_FORWARD_LEFT 25
#define M_FORWARD_RIGHT 26
#define M_BACKWARD_LEFT 27
#define M_BACKWARD_RIGHT 28

#define F_FORWARD 31
#define F_BACKWARD 32
#define F_LEFT 33
#define F_RIGHT 34
#define F_FORWARD_LEFT 35
#define F_FORWARD_RIGHT 36
#define F_BACKWARD_LEFT 37
#define F_BACKWARD_RIGHT 38

#define STOP 0
#define START 1

smartcarContol car;
AsyncWebServer server(80);
AsyncWebSocket ws("/ws");

extern void web_init(void);
/* Wifi Crdentials */
const char* sta_ssid = "PPGG";     // set Wifi network you want to connect to
const char* sta_password = "clippinglp1";        // set password for Wifi network
unsigned long previousMillis = 0;
void init_wifi(void)
{
  String hostname = "esp32brobot";
  // first, set NodeMCU as STA mode to connect with a Wifi network
  //WiFi.mode(WIFI_STA);
  //WiFi.begin(sta_ssid, sta_password);
  WiFi.softAP(sta_ssid, sta_password);
  IPAddress IP = WiFi.softAPIP();
  Serial.print("AP IP address: ");
  Serial.println("");
  Serial.print("Connecting to: ");
  Serial.println(sta_ssid);
  Serial.print("Password: ");
  Serial.println(sta_password);
  // try to connect with Wifi network about 8 seconds
  unsigned long currentMillis = millis();
  previousMillis = currentMillis;
/*  while (WiFi.status() != WL_CONNECTED && currentMillis - previousMillis <= 8000) {
    delay(500);
    Serial.print(".");
    currentMillis = millis();
  }*/
  // if failed to connect with Wifi network set NodeMCU as AP mode
  IPAddress myIP;
  if (WiFi.status() == WL_CONNECTED) {
    Serial.println("");
    Serial.println("*WiFi-STA-Mode*");
    Serial.print("IP: ");
    myIP=WiFi.localIP();
    Serial.println(myIP);
    delay(2000);
  } else {
    WiFi.mode(WIFI_AP);
    WiFi.softAP(hostname.c_str());
    myIP = WiFi.softAPIP();
    Serial.println("");
    Serial.println("WiFi failed connected to ");
    Serial.println("");
    Serial.println("*WiFi-AP-Mode*");
    Serial.print("AP IP address: ");
    Serial.println(myIP);
    delay(2000);
  }

}

void onWebSocketEvent(AsyncWebSocket * server, AsyncWebSocketClient * client, AwsEventType type, void * arg, uint8_t *data, size_t len);
void web_init(void)
{
  init_wifi();
  ws.onEvent(onWebSocketEvent);
  server.addHandler(&ws);

  server.begin();
  Serial.println("HTTP service started");
}

void processCarMovement(String inputValue)
{
  Serial.printf("Got value as %s %d\n", inputValue.c_str(), inputValue.toInt());
  int getcontrol = inputValue.toInt();
  int getspeed = getcontrol/10;
  int getDiretion = getcontrol%10;
  switch (getDiretion)
  {

    case FORWARD:
      car.car_forward();
      Serial.println("car_forward");
      break;

    case BACKWARD:
      car.car_reverse();
      break;

    case LEFT:
      car.car_turnLeft();
      break;

    case RIGHT:
      car.car_turnRight();
      break;

    case FORWARD_LEFT:
      car.car_turnLeftCycle();
      break;
    case FORWARD_RIGHT:
      car.car_turnRightCycle();
      break;

    case BACKWARD_LEFT:
      car.car_sotp();
      break;

    case S_BACKWARD_RIGHT:

      break;







    case STOP:

      break;

    default:

      break;
  }
}


void onWebSocketEvent(AsyncWebSocket * server, AsyncWebSocketClient * client, AwsEventType type, void * arg, uint8_t *data, size_t len)
{
  switch (type)
  {
    case WS_EVT_CONNECT:
      Serial.printf("WebSocket client #%u connected from %s\n", client->id(), client->remoteIP().toString().c_str());
      ws.text(client->id(), String("connected"));
      break;
    case WS_EVT_DISCONNECT:
      Serial.printf("WebSocket client #%u disconnected\n", client->id());
      processCarMovement("0");
      break;
    case WS_EVT_DATA:
      AwsFrameInfo *info;
      info = (AwsFrameInfo*)arg;
      if (info->final && info->index == 0 && info->len == len && info->opcode == WS_TEXT)
      {
        std::string myData = "";
        myData.assign((char *)data, len);
        processCarMovement(myData.c_str());
      }
      break;
    case WS_EVT_PONG:
    case WS_EVT_ERROR:
      break;
    default:
      break;
  }
}