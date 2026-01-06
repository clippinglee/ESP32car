
#ifndef __CAR_CONTROL__
#define __CAR_CONTROL__
#include "L298N.h"

#define FORWARD 1
#define BACKWARD 2
#define LEFT 3
#define RIGHT 4
#define LEFT_CYCLE 5
#define RIGHT_CYCLE 6
#define STOP 7
class smartcarContol{
    public:
    void init(void);
    static uint8_t comunicate_connected;
    static uint8_t status;
    static uint8_t laststatus;
    void car_stop(void);
    void car_forward(void);
    void car_turnRight(void);
    void car_turnLeft(void);
    void car_reverse(void);
    void car_turnLeftCycle(void);
    void car_turnRightCycle(void);
    private:
    L298N_t* motor1;
    L298N_t* motor2;
};

#endif /*__CAR_CONTROL__*/