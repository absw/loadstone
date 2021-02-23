# Pmod ESP32 Example TCP Session

The default serial port parameters for the esp32 are:
115200 baud, 8 bits, no parity

All commands should be terminated with cr-lf (^M^J)

# Useful References

[reference manual](https://reference.digilentinc.com/reference/pmod/pmodesp32/start)
[at command set](https://docs.espressif.com/projects/esp-at/en/latest/AT_Command_Set/index.html)

# Initial Configuration
* Configuration of the ESP32 SoftAP
Command:
AT+CWSAP="LoadStone","",1,0
Response:
^M^J
OK^M^J

* Set the Wi-Fi Mode
Command:
AT+CWMODE=2 (1=station, 2=softAP default gateway=192.168.4.1)
Response:
^M^J
OK^M^J

Note: The SoftAP can be configured offline, and is saved:

# Running Configuration
* AT Commands Echoing
Command:
ATE0	(switches echo off)
Response:
^M^J
OK^M^J

# Run a TCP Server:
* Enable/Disable Multiple Connections
Command:
AT+CIPMUX=1	(multiple connections)
Response:
^M^J
OK^M^J

* Set the Maximum Connections Allowed by Server
Command:
AT+CIPSERVERMAXCONN=1
Response:
^M^J
OK^M^J

* Delete/Create TCP Server
Command:
AT+CIPSERVER=1,9999
Response:
^M^J
OK^M^J

* When a PC/Phone connects to the SoftAP:
Response:
+STA_CONNECTED:"24:ee:9a:2d:6e:b4"^M^J
+DIST_STA_IP:"24:ee:9a:2d:6e:b4","192.168.4.2"^M^J

* When a PC/Phone connects to TCP socket 9999:
0,CONNECT^M^J

# Data Transfer after connectoin
* When data is received:
Response:
+IPD,0,4:test^M^J	(0=channel, 4=number of chars)

* To send data:
Command:
AT+CIPSEND=0,5	(0=channel, 5=number of characters)
Response:
^M^J
OK^M^J
^M^J
>
12345	(data to send)
Response:
^M^J
Recv 5 bytes^M^J
^M^J
SEND OK^M^J

# When TCP socket is closed:
Response:
0,CLOSED^M^J

# When a PC/Phone Disconnects from the SoftAP
+STA_DISCONNECTED:"24:ee:9a:2d:6e:b4"^M^J
