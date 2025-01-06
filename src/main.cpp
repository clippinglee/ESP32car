#include <Arduino.h>
#include <Arduino.h>
#include "CarControl/CarControl.hpp"
// put function declarations here:
int myFunction(int, int);

extern void ble_init(void);
void setup() {
  pinMode(2, OUTPUT);
  pinMode(14, OUTPUT);
  digitalWrite(14,0);
  pinMode(27, OUTPUT);
  digitalWrite(27,0);
  pinMode(12, OUTPUT);
  digitalWrite(12,0);
  pinMode(13, OUTPUT);
  digitalWrite(13,0);
  //ledcSetup(6, 50, 16); // channel 6, 50 Hz, 16-bit width
  Serial.begin(115200);         // set up seriamonitor at 115200 bps
  Serial.setDebugOutput(true);
  Serial.println();
  Serial.println("*ESP32 samrt car*");
  Serial.println("--------------------------------------------------------");
  ble_init();
}
void led_blink(uint8_t channnel)
{
    static uint8_t led_status;
    if(led_status == 0)
    {
        digitalWrite(channnel,1);
        led_status = 1;
    }
    else
    {
        digitalWrite(channnel,0);
        led_status = 0;
    }
    //Serial.printf("LED STATUS %d\n",led_status);
}
extern void ble_main(void);
smartcarContol car1;
void loop() {
  // put your main code here, to run repeatedly:
  car1.car_forward();
  led_blink(2);
  //digitalWrite(2,1);
  delay(500);
  ble_main();
}

