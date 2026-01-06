
#include "CarControl.hpp"
uint8_t smartcarContol::comunicate_connected = 0;
uint8_t smartcarContol::status = 0;
uint8_t smartcarContol::laststatus = 0;

void smartcarContol::init(void){

    L298N_PinType_t motor1Cfg = {16,17,NULL,-1,NULL};  //不使用pwm功能    
    L298N_PinType_t motor2Cfg = {18,19,NULL,-1,NULL};  //不使用pwm功能
    Serial.println("-----------------smartcarContol----------------------------------");
    motor1 = L298N_Create(&motor1Cfg);
    motor2 = L298N_Create(&motor2Cfg);   
    if ((motor2 == NULL)||(motor1 == NULL))
    {

        while(1)
        {
                        Serial.println((uint32_t)motor1, HEX);  // 以十六进制打印地址
        Serial.println((uint32_t)motor2, HEX);
            delay(1);
        }
    }
    
}


void smartcarContol::car_turnRight(void)
{
    motor1->Control(motor1,MOTOR_FORWARD);
    motor2->Control(motor2,MOTOR_STOP);
    status = RIGHT;
}

void smartcarContol::car_turnLeft(void)
{
    motor1->Control(motor1,MOTOR_STOP);
    motor2->Control(motor2,MOTOR_FORWARD);
    status = LEFT;
}

void smartcarContol::car_forward(void)
{
    motor1->Control(motor1,MOTOR_FORWARD);
    motor2->Control(motor2,MOTOR_FORWARD);
    status = FORWARD;
}

void smartcarContol::car_stop(void)
{
    motor1->Control(motor1,MOTOR_STOP);
    motor2->Control(motor2,MOTOR_STOP);
    status = STOP;
}


void smartcarContol::car_reverse(void)
{
    motor1->Control(motor1,MOTOR_REVERSE);
    motor2->Control(motor2,MOTOR_REVERSE);
    status = BACKWARD;
}

void smartcarContol::car_turnLeftCycle(void)
{
    motor1->Control(motor1,MOTOR_REVERSE);
    motor2->Control(motor2,MOTOR_FORWARD);
    status = LEFT_CYCLE;
}

void smartcarContol::car_turnRightCycle(void)
{
    motor1->Control(motor1,MOTOR_FORWARD);
    motor2->Control(motor2,MOTOR_REVERSE);
    status = RIGHT_CYCLE;
}
