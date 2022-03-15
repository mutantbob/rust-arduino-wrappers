#include <SPI.h>

unsigned char SPI_transfer()
{
    return SPI.transfer(0);
}
