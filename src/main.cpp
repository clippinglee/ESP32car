#include <Arduino.h>
#include "CarControl/CarControl.hpp"
// put function declarations here:
int myFunction(int, int);
void carmotion(void *pvParameters) ;
extern void wifi_init();
extern void car_impl(smartcarContol* impl);
smartcarContol car1;
void setup() {
  pinMode(2, OUTPUT);
  pinMode(26, OUTPUT);
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
  car1.comunicate_connected = 0;
  car1.init();
  xTaskCreatePinnedToCore(carmotion, "carmotion", 10000, NULL, 5, NULL, 0);
  wifi_init();
  car_impl(&car1);
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

void loop() {
  // put your main code here, to run repeatedly:

  led_blink(2);
  //digitalWrite(2,1);
  delay(500);

}

uint32_t timecnt;
void carmotion(void *pvParameters) {
  Serial.println("-----------------carmotion----------------------------------");
  while (1) {
    timecnt++;
    if(car1.comunicate_connected == 0)
    {   
        if(timecnt < 10)
            car1.car_forward();
        else if(timecnt < 20)
            car1.car_reverse();
        else if(timecnt < 30)
            car1.car_turnLeftCycle();
        else if(timecnt < 40)
            car1.car_stop();
        else
            timecnt = 0;
        delay(500);
    }
    else
    {
        if((car1.status == LEFT) || (car1.status == RIGHT))
        {
            delay(200);
            if(car1.laststatus ==  FORWARD)
            {
                car1.car_forward();
            }
            else if (car1.laststatus ==  BACKWARD)
            {
                car1.car_reverse();
            }
            else
            {
                car1.car_stop();
            }

        }
    }
    
  }
}