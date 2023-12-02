import sys
import serial 
from Crypto.Cipher import AES

# Choose what serial port your board is on
# - For linux: /dev/ttyUSB1, /dev/ttyUSB2, etc...
# - For windows: COM1, COM2, etc...  or something like that I don't really know
SERIAL_PORT = "/dev/ttyUSB1"

def pkcs7(data, block_size=16):
    pl = block_size - (len(data) % block_size)
    return data + bytearray([pl for i in range(pl)])

# --- Execution starts here ---

usage = """usage: {0} [-b | -l] hex_key file
    -b: Use board
    -l: Use AES library (no trojan)
    hex_key: 16 byte key for AES in hexadecimal
    file: Input file
    """

if len(sys.argv) != 4:
    print(usage.format(sys.argv[0]))
    print("note: you must click the center button if you want the board to take a new key")
    sys.exit()

use_board = None
if sys.argv[1] not in {"-b", "-l"}:
    print("Unrecognized flag {0}, use -b or -l".format(sys.argv[1]))
    sys.exit()
else:
    if sys.argv[1] == "-b":
        use_board = True
    else:
        use_board = False

if len(sys.argv[2]) != 32:
    print("Length of the key must be 16 bytes! (32 hex characters)")
    sys.exit()

with open(sys.argv[3], "rb") as in_file:
    # encrypt using the board
    if (use_board):
        ser = serial.Serial(SERIAL_PORT)

        # Write key to board
        ser.write(bytes.fromhex(sys.argv[2]))
        ser.read(16)

        while True:
            data = in_file.read(16)
            if len(data) == 16:
                ser.write(data)
                print(ser.read(16).hex())
            else:
                ser.write(pkcs7(data))
                print(ser.read(16).hex())
                break
    # encrypt using an AES library
    else:
        cipher = AES.new(bytes.fromhex(sys.argv[2]), AES.MODE_ECB)
        while True:
            data = in_file.read(16)
            if len(data) == 16:
                print(cipher.encrypt(data).hex())
            else:
                print(cipher.encrypt(pkcs7(data)).hex())
                break
