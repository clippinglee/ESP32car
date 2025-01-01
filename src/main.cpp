#include <Arduino.h>
#include "CarControl/CarControl.hpp"
// put function declarations here:
int myFunction(int, int);

void setup() {
  pinMode(2, OUTPUT);
  pinMode(16, OUTPUT);
  pinMode(17, OUTPUT);
  pinMode(18, OUTPUT);
  pinMode(19, OUTPUT);
  Serial.begin(115200);

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
    Serial.printf("LED STATUS %d\n",led_status);
}

void loop() {
  // put your main code here, to run repeatedly:
  smartcarContol car;
  car.car_Forward();
  led_blink(2);
  delay(500);
}

