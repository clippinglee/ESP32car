#include "L298N.h"
#include "CarControl.hpp"

void smartcarContol::car_turnRight(void)
{
    L298N_MotorControl(0,MOTOR_FORWARD);
    L298N_MotorControl(1,MOTOR_STOP);
}

void smartcarContol::car_turnLeft(void)
{
    L298N_MotorControl(0,MOTOR_STOP);
    L298N_MotorControl(1,MOTOR_FORWARD);
}

void smartcarContol::car_forward(void)
{
    L298N_MotorControl(0,MOTOR_FORWARD);
    L298N_MotorControl(1,MOTOR_FORWARD);
}


void smartcarContol::car_reverse(void)
{
    L298N_MotorControl(0,MOTOR_REVERSE);
    L298N_MotorControl(1,MOTOR_REVERSE);
}

void smartcarContol::car_turnLeftCycle(void)
{
    L298N_MotorControl(0,MOTOR_REVERSE);
    L298N_MotorControl(1,MOTOR_FORWARD);
}

void smartcarContol::car_turnRightCycle(void)
{
    L298N_MotorControl(0,MOTOR_FORWARD);
    L298N_MotorControl(1,MOTOR_REVERSE);
}
