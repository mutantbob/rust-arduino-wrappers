#include <Arduino.h>
#include <SPI.h>
#include <Ethernet.h>

EthernetServer fabricate_EthernetServer(uint16_t port);
void virtual_EthernetServer_begin(EthernetServer* that);
bool virtual_EthernetClient_connected(EthernetClient* that);
int virtual_EthernetClient_available(EthernetClient* that);
int virtual_EthernetClient_read(EthernetClient* that);
size_t virtual_EthernetClient_println(EthernetClient* that, const unsigned char* msg);
void virtual_EthernetClient_stop(EthernetClient *that);
bool EthernetClient_valid(const EthernetClient *that);
int virtual_EthernetClient_availableForWrite(EthernetClient* that);
