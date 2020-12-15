Code is experimental untill it hits 1.x version.

### What is working:
* reading fuses
* reading signature
* reading lock byte
* reading EEPROM

### Help wanted
AVR devices have their quirks. For now devices with < 64K of flash can be read.
For devices with > 64K flash, AVR programmers have some kind of different procedure.
If you know how it is implemented in e.g. `avrdude`, please help.
