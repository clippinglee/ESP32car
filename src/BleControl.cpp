
#include <Arduino.h>
#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEServer.h>
#include <BLE2902.h>

#include "CarControl/CarControl.hpp"


#define FORWARD 1
#define BACKWARD 2
#define LEFT 3
#define RIGHT 4
#define LEFT_CYCLE 5
#define RIGHT_CYCLE 6
#define STOP 7
#define ACC 8


BLEServer* pServer = NULL;
BLECharacteristic* pCharacteristic = NULL;

#define SERVICE_UUID        "4fafc201-1fb5-459e-8fcc-c5c9c331914b"
#define CHARACTERISTIC_UUID "beb5483e-36e1-4688-b7f5-ea07361b26a8"

bool deviceConnected = false;
bool oldDeviceConnected = false;

static BLERemoteCharacteristic* pRemoteCharacteristic;
static BLEAdvertisedDevice* myDevice;
smartcarContol car;

static void notifyCallback(
  BLERemoteCharacteristic* pBLERemoteCharacteristic,
  uint8_t* pData,
  size_t length,
  bool isNotify) {
    Serial.print("Notify callback for characteristic ");
    Serial.print(pBLERemoteCharacteristic->getUUID().toString().c_str());
    Serial.print(" of data length ");
    Serial.println(length);
    Serial.print("data: ");
    Serial.println((char*)pData);
}

class MyServerCallbacks: public BLEServerCallbacks {
    void onConnect(BLEServer* pServer) {
        deviceConnected = true;
    };

    void onDisconnect(BLEServer* pServer) {
        deviceConnected = false;
    }
};
void processCarMovement(std::string inputValue);
class charCallback:public BLECharacteristicCallbacks{
    void onWrite(BLECharacteristic *pCharacteristic) {
        std::string value = pCharacteristic->getValue();
        if(value.length() > 0) {
            Serial.println(value.c_str());
            processCarMovement(value);
        }
    }
};

extern void ble_init(void);
void ble_init(void)
{
 // Create the BLE Device
  BLEDevice::init("ESP32");

  pServer = BLEDevice::createServer();
  pServer->setCallbacks(new MyServerCallbacks());
  BLEService *pService = pServer->createService(SERVICE_UUID);
  pCharacteristic = pService->createCharacteristic(
                                         CHARACTERISTIC_UUID,
                                         BLECharacteristic::PROPERTY_READ |
                                         BLECharacteristic::PROPERTY_WRITE|
                                         BLECharacteristic::PROPERTY_NOTIFY |
                                         BLECharacteristic::PROPERTY_INDICATE
                                       );
  pCharacteristic->setCallbacks(new charCallback());
  pCharacteristic->setValue("Hello World says Neil");
  pCharacteristic->addDescriptor(new BLE2902());
  pService->start();
  // BLEAdvertising *pAdvertising = pServer->getAdvertising();  // this still is working for backward compatibility
  BLEAdvertising *pAdvertising = BLEDevice::getAdvertising();
  pAdvertising->addServiceUUID(SERVICE_UUID);
  pAdvertising->setScanResponse(true);
  pAdvertising->setMinPreferred(0x06);  // functions that help with iPhone connections issue
  pAdvertising->setMinPreferred(0x12);
  BLEDevice::startAdvertising();
  Serial.println("Characteristic defined! Now you can read it in your phone!");
}

void processCarMovement(std::string inputValue)
{
  Serial.printf("Got value as %s %d\n", inputValue.c_str(),  std::stoi(inputValue));
  int getcontrol = std::stoi(inputValue);
  int getspeed = getcontrol/10;
  int getDiretion = getcontrol%10;
  car.comunicate_connected = 1;
  switch (getDiretion)
  {

    case FORWARD:
      car.car_forward();
      Serial.println("car_forward");
      break;

    case BACKWARD:
      car.car_reverse();
      Serial.println("car_reverse");
      break;

    case LEFT:
      car.car_turnLeft();
      Serial.println("car_turnLeft");
      break;

    case RIGHT:
      car.car_turnRight();
      Serial.println("car_turnRight");
      break;

    case LEFT_CYCLE:
      car.car_turnLeftCycle();
      Serial.println("car_turnLeftCycle");
      break;
    case RIGHT_CYCLE:
      car.car_turnRightCycle();
      Serial.println("car_turnRightCycle");
      break;

    case STOP:
      car.car_sotp();
      Serial.println("car_sotp");
      break;
    default:

      break;
  }
}
extern void ble_main(void);
void ble_main(void)
{

    // disconnecting
    if (!deviceConnected && oldDeviceConnected) {
        delay(500); // give the bluetooth stack the chance to get things ready
        pServer->startAdvertising(); // restart advertising
        Serial.println("start advertising");
        oldDeviceConnected = deviceConnected;
    }
    // connecting
    if (deviceConnected && !oldDeviceConnected) {
        // do stuff here on connecting
        oldDeviceConnected = deviceConnected;
    }
}