#include "Ethernet.cpp"


// bindgen does not generate anything for the inline constructors
EthernetServer fabricate_EthernetServer( uint16_t port)
{
    return EthernetServer(port);
}

void virtual_EthernetServer_begin(EthernetServer* that)
{
     that -> begin();
}

//

EthernetClient fabricate_EthernetClient()
{
    return EthernetClient();
}

int virtual_EthernetClient_connect_hostname(EthernetClient* that, const char *host, uint16_t port)
{
    return that->connect(host, port);
}

int virtual_EthernetClient_availableForWrite(EthernetClient* that)
{
    return that->availableForWrite();
}

bool virtual_EthernetClient_connected(EthernetClient* that)
{
    return that->connected();
}

int virtual_EthernetClient_available(EthernetClient* that)
{
    return that->available();
}

size_t virtual_EthernetClient_write(EthernetClient* that, const uint8_t *buf, size_t size)
{
    return that->write(buf, size);
}

int virtual_EthernetClient_read(EthernetClient* that)
{
    return that->read();
}

int virtual_EthernetClient_readMulti(EthernetClient* that, uint8_t* buffer, size_t size)
{
    return that->read(buffer, size);
}

size_t virtual_EthernetClient_println(EthernetClient* that, const unsigned char* msg)
{
    return that->println((const char *)msg);
}

void virtual_EthernetClient_flush(EthernetClient *that)
{
    that->flush();
}

void virtual_EthernetClient_stop(EthernetClient *that)
{
    that->stop();
}

bool EthernetClient_valid(const EthernetClient *that)
{
    return *(EthernetClient*)that;
}

Client* cast_to_Client(EthernetClient *that)
{
    return (Client*)that;
}

IPAddress virtual_EthernetClient_remoteIP(const EthernetClient *that)
{
    return ((EthernetClient*)that)->remoteIP();
}

EthernetUDP fabricate_EthernetUDP()
{
    return EthernetUDP();
}
