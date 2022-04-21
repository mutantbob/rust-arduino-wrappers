#include <Arduino.h>
#include <SPI.h>
#include <Ethernet.h>
#include <Dns.h>

EthernetServer fabricate_EthernetServer(uint16_t port);
void virtual_EthernetServer_begin(EthernetServer* that);
//
EthernetClient fabricate_EthernetClient();
int virtual_EthernetClient_connect_hostname(EthernetClient* that, const char *host, uint16_t port);
bool virtual_EthernetClient_connected(EthernetClient* that);
int virtual_EthernetClient_available(EthernetClient* that);
size_t virtual_EthernetClient_write(EthernetClient* that, const uint8_t *buf, size_t size);
int virtual_EthernetClient_read(EthernetClient* that);
int virtual_EthernetClient_readMulti(EthernetClient* that, uint8_t* buffer, size_t size);
size_t virtual_EthernetClient_println(EthernetClient* that, const unsigned char* msg);
void virtual_EthernetClient_flush(EthernetClient *that);
void virtual_EthernetClient_stop(EthernetClient *that);
bool EthernetClient_valid(const EthernetClient *that);
int virtual_EthernetClient_availableForWrite(EthernetClient* that);
Client* cast_to_Client(EthernetClient *that);

IPAddress virtual_EthernetClient_remoteIP(const EthernetClient *that);

//

EthernetUDP fabricate_EthernetUDP();
