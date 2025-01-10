#include "L298N.h"
#include "CarControl.hpp"
uint8_t smartcarContol::comunicate_connected = 0;
uint8_t smartcarContol::status = 0;
uint8_t smartcarContol::laststatus = 0;
void smartcarContol::car_turnRight(void)
{
    
    L298N_MotorControl(0,MOTOR_FORWARD);
    L298N_MotorControl(1,MOTOR_STOP);
    status = RIGHT;
}

void smartcarContol::car_turnLeft(void)
{
    L298N_MotorControl(0,MOTOR_STOP);
    L298N_MotorControl(1,MOTOR_FORWARD);
    status = LEFT;
}

void smartcarContol::car_forward(void)
{
    L298N_MotorControl(0,MOTOR_FORWARD);
    L298N_MotorControl(1,MOTOR_FORWARD);
    status = FORWARD;
}

void smartcarContol::car_stop(void)
{
    L298N_MotorControl(0,MOTOR_STOP);
    L298N_MotorControl(1,MOTOR_STOP);
    status = STOP;
}


void smartcarContol::car_reverse(void)
{
    L298N_MotorControl(0,MOTOR_REVERSE);
    L298N_MotorControl(1,MOTOR_REVERSE);
    status = BACKWARD;
}

void smartcarContol::car_turnLeftCycle(void)
{
    L298N_MotorControl(0,MOTOR_REVERSE);
    L298N_MotorControl(1,MOTOR_FORWARD);
    status = LEFT_CYCLE;
}

void smartcarContol::car_turnRightCycle(void)
{
    L298N_MotorControl(0,MOTOR_FORWARD);
    L298N_MotorControl(1,MOTOR_REVERSE);
    status = RIGHT_CYCLE;
}
