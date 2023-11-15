#!/usr/bin/python
# HIGHLEVEL_CLASSLOAD_FAIL_FUNC_DEBUG
# -*- coding: utf-8 -*-
#
# Copyright (c) 2013-2021, NewAE Technology Inc
# All rights reserved.
#
# Find this and more at newae.com - this file is part of the chipwhisperer
# project, http://www.assembla.com/spaces/chipwhisperer
#
#    This file is part of chipwhisperer.
#
#    Licensed under the Apache License, Version 2.0 (the "License");
#    you may not use this file except in compliance with the License.
#    You may obtain a copy of the License at
#
#       http://www.apache.org/licenses/LICENSE-2.0
#
#    Unless required by applicable law or agreed to in writing, software
#    distributed under the License is distributed on an "AS IS" BASIS,
#    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#    See the License for the specific language governing permissions and
#    limitations under the License.
#=================================================
import logging
import time

import chipwhisperer.capture.scopes._qt as openadc_qt # type: ignore
from ._base import TargetTemplate
#from chipwhisperer.capture.scopes.openadc_interface import ftdi
from chipwhisperer.common.utils import util
#from chipwhisperer.common.utils.parameter import Parameterized, Parameter, setupSetParam

from chipwhisperer.logging import *

try:
    import ftd2xx as ft # type: ignore
except OSError:  # Also catches WindowsError
    raise ImportError


class ChipWhispererComm:
    _name = "Sakurag"
    CODE_READ = 0x80
    CODE_WRITE = 0xC0

    ADDR_STATUS = 49
    ADDR_FIFO = 50

    FLAG_RESET = 0x01
    FLAG_WRFULL = 0x02
    FLAG_RDEMPTY = 0x04

    def __init__(self, standalone=False):
        self.standalone = standalone
        self.serialnum = None

        if standalone:
            self.setSerial = self._setSerial

    def _setSerial(self, serialnum):
        self.serialnum = serialnum

    def reset(self):
        self.oa.sendMessage(self.CODE_WRITE, self.ADDR_STATUS, [self.FLAG_RESET], Validate=False)
        time.sleep(0.05)
        self.oa.sendMessage(self.CODE_WRITE, self.ADDR_STATUS, [0x00], Validate=False)

    def con(self, scope=None):
        if scope and scope.qtadc and scope.qtadc.sc:
            self.oa = scope.qtadc.sc
        else:
            if self.serialnum is not None:
                self.qtadc = openadc_qt.OpenADCQt()
                self.params.append(self.qtadc.getParams())
                self.oaiface = ftdi.OpenADCInterface_FTDI(None, self.qtadc)
                self.params.append(self.oaiface.getParams())
                self.oaiface.setSerialNumberLimits([self.serialnum])
                self.oaiface.setSelectedDevice(self.serialnum)
                self.oaiface.con()
                self.oa = self.qtadc.sc
            else:
                raise Warning("No OpenADC detected  - did you connect the scope module and/or set serial number (hit 'REFRESH')?")

        # Reset AES Core
        self.oa.sendMessage(self.CODE_WRITE, self.ADDR_STATUS, [self.FLAG_RESET], Validate=False)
        self.oa.sendMessage(self.CODE_WRITE, self.ADDR_STATUS, [0x00], Validate=False)
        return True

    def disconnect(self):
        return

    def flush(self):
        while (self.readStatus() & self.FLAG_RDEMPTY) != self.FLAG_RDEMPTY:
            self.oa.sendMessage(self.CODE_READ, self.ADDR_FIFO, Validate=False)

    def readStatus(self):
        b = self.oa.sendMessage(self.CODE_READ, self.ADDR_STATUS, Validate=False)
        return b[0]

    def writeMsg(self, msg):

        for b in msg:
            # while (self.readStatus() & self.FLAG_WRFULL) == self.FLAG_WRFULL:
            #    pass

            self.oa.sendMessage(self.CODE_WRITE, self.ADDR_FIFO, [b], Validate=False)

    def readMsg(self, nbytes):
        msg = bytearray()

        for i in range(0, nbytes):
            if self.readStatus() & self.FLAG_RDEMPTY:
                pass

            b = self.oa.sendMessage(self.CODE_READ, self.ADDR_FIFO, Validate=False)
            msg.append(b[0])

        return msg

    def write(self, address, MSB, LSB):
        msg = bytearray(5)

        msg[0] = 0x01
        msg[1] = (address >> 8) & 0xFF  # MSB
        msg[2] = address & 0xFF  # LSB
        msg[3] = MSB
        msg[4] = LSB

        # msg = bytearray(strmsg)
        # print "Write: %x %x %x %x %x"%(msg[0],msg[1],msg[2],msg[3],msg[4])

        self.writeMsg(msg)

    def read(self, address):
        self.flush()
        msg = bytearray(3)
        msg[0] = 0x00
        msg[1] = (address >> 8) & 0xFF  # MSB
        msg[2] = address & 0xFF  # LSB
        self.writeMsg(msg)
        # print "Write: %x %x %x"%(msg[0],msg[1],msg[2]),

        msg = self.readMsg(2)

        # print " Read: %x %x"%(msg[0],msg[1])

        # Order = MSB, LSB
        return msg

    def read128(self, address):
        self.flush()
        msg = bytearray(3 * 8)
        for i in range(0, 8):
            msg[i * 3] = 0x00
            msg[i * 3 + 1] = (address >> 8) & 0xFF
            msg[i * 3 + 2] = (address & 0xFF) + (i * 2)
        self.writeMsg(bytes(msg))
        msg = self.readMsg(16)
        return bytearray(msg)

    def close(self):
        pass


class SakuraG(TargetTemplate):
    _name = "SAKURA G"

    def __init__(self):
        TargetTemplate.__init__(self)
        self.hw = None

        # conntypes = util.DictType()
        # conntypes['Select Interface type...'] = None
        # conntypes['CW Bitstream, with OpenADC'] = ChipWhispererComm(standalone=False)
        # conntypes['CW Bitstream, no OpenADC'] = ChipWhispererComm(standalone=True)
        # conntypes['Original Bitstream'] = FTDIComm()
        self.hw = ChipWhispererComm(standalone=False)

        self.fixedStart = True
        target_logger.warning("SAKURA-G support is currently in an alpha state - errors may occur")
        # self.hw = None

    def getConn(self):
        return self.hw

    def setConn(self, con):
        self.hw = con
        # if hasattr(self.hw, 'setSerial'):
        #     self.findParam('serno').show()
        #     self.findParam('pushsno').show()
        # else:
        #     self.findParam('serno').hide()
        #     self.findParam('serno').hide()

    def refreshSerial(self, _=None):
        return None
        # serialnames = ft.listDevices()
        # if serialnames == None:
        #     serialnames = [" No Connected Devices "]

        # self.findParam('serno').setLimits(serialnames)
        # self.findParam('serno').setValue(serialnames[0])

    def _con(self, scope=None):
        # self.hw = self.findParam('conn').getValue()
        self.hw.con(scope)
        self.init()
        return

        if hasattr(self.hw, 'setSerial'):
            # For SAKURA-G normally we use 'A' channel
            # ser = self.findParam('serno').getValue()
            # if ser.endswith('A') is False:
            #     target_logger.warning('Normally SAKURA-G uses "A" ending in serial number')
            self.hw.setSerial(ser)

        if self.hw.con(scope):
            self.init()
            self.setModeEncrypt()
        else:
            raise Warning("Opened USB, but failed to find expected FPGA setup")

    def reset(self, _=None):
        if self.hw:
            if hasattr(self.hw, 'reset'):
                self.hw.reset()

    def disconnect(self):
        if self.hw:
            self.hw.disconnect()

    def close(self):
        if self.hw:
            self.hw.close()

    def init(self):
        #Select AES
        self.hw.write(0x0004, 0x00, 0x01)
        self.hw.write(0x0006, 0x00, 0x00)

        #Reset AES module
        self.hw.write(0x0002, 0x00, 0x04)
        self.hw.write(0x0002, 0x00, 0x00)

        #Select AES output
        self.hw.write(0x0008, 0x00, 0x01)
        self.hw.write(0x000A, 0x00, 0x00)

    def setModeEncrypt(self):
        self.hw.write(0x000C, 0x00, 0x00)

    def setModeDecrypt(self):
        self.hw.write(0x000C, 0x00, 0x01)

    def checkEncryptionKey(self, key):
        #SAKURA-G AES Example has fixed first 9 bytes
        if self.fixedStart:
            for i in range(0,9):
                key[i] = i                
        return key 

    def loadEncryptionKey(self, key):
        """Encryption key is bytearray"""

        if key:                
            self.hw.write(0x0100, key[0], key[1])
            self.hw.write(0x0102, key[2], key[3])
            self.hw.write(0x0104, key[4], key[5])
            self.hw.write(0x0106, key[6], key[7])
            self.hw.write(0x0108, key[8], key[9])
            self.hw.write(0x010A, key[10], key[11])
            self.hw.write(0x010C, key[12], key[13])
            self.hw.write(0x010E, key[14], key[15])

        #Generate key schedule
        self.hw.write(0x0002, 0x00, 0x02)

        #Wait for done
        while self.isDone() == False:
            continue

    def loadInput(self, inputtext):
        self.hw.write(0x0140, inputtext[0], inputtext[1])
        self.hw.write(0x0142, inputtext[2], inputtext[3])
        self.hw.write(0x0144, inputtext[4], inputtext[5])
        self.hw.write(0x0146, inputtext[6], inputtext[7])
        self.hw.write(0x0148, inputtext[8], inputtext[9])
        self.hw.write(0x014A, inputtext[10], inputtext[11])
        self.hw.write(0x014C, inputtext[12], inputtext[13])
        self.hw.write(0x014E, inputtext[14], inputtext[15])

    def is_done(self):
        result = self.hw.read(0x0002)

        if result[0] == 0x00: # and result[1] == 0x00:
            return True
        else:
            return False

    def readOutput(self):        
        return self.hw.read128(0x0180)

    def setMode(self, mode):
        if mode == "encryption":
            self.hw.write(0x000C, 0x00, 0x00)
        elif mode == "decryption":
            self.hw.write(0x000C, 0x00, 0x01)
        else:
            raise ValueError

    def go(self):
        self.hw.write(0x0002, 0x00, 0x01)

    def simpleserial_write(self, cmd, data, end=None):
        if cmd == 'p':
            self.loadInput(data)
            self.go()
        if cmd == 'k':
            self.loadEncryptionKey(data)
        pass
    def simpleserial_read(self, cmd, pay_len, end='\n', timeout=250, ack=True):
        if cmd == 'r':
            return self.readOutput()
        pass

    def set_key(self, key, ack=False):
        self.simpleserial_write('k', key)