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
    return *that;
}
