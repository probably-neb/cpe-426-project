This repository contains Verilog modules that are common to multiple NewAE
projects.

# Notes regarding FIFOs
These FIFOs were developed to overcome issues with simulating the Xilinx FIFOs
with the Icarus Verilog simulator. They are not (yet) intended to be a full
replacement for Xilinx FIFOs in implementation. Tests show that the Xilinx
FIFOs result in better resource utilization and timing closure. However,
these FIFOs simulate *much* faster than the Xilinx ones.

The [fifo\_sync](hdl/fifo_sync.v) module aims to have the exact same
behaviour as the Xilinx synchronous FIFO, and while there is some limited
testing for this, this is not exhaustively tested or proven (yet).

The [fifo\_async](hdl/fifo_async.v) currently does *not* have the same
behaviour as the Xilinx asynchronous FIFO (i.e. in terms of timing of its
empty/full status signals).


# Notes regarding SS2 wrapper
[ss2.v](hdl/ss2.v) implements the [SimpleSerial
v2.1](https://chipwhisperer.readthedocs.io/en/latest/simpleserial.html#simpleserial-v2-1)
protocol (SS2) and maps SS2 commands to CW305 register read/writes. This allows
existing CW305 design which use the SAM3U USB parallel interface to be
very easily ported to the CW312T-A35 and CW312T-iCE40 target boards. 

Examples exist
for our existing [CW305 AES](https://github.com/newaetech/chipwhisperer/blob/develop/hardware/victims/cw308_ufo_target/xc7a35/vivado/ss2_cw305_aes.xpr), 
[CW305 AES pipeline](https://github.com/newaetech/chipwhisperer/blob/develop/hardware/victims/cw308_ufo_target/xc7a35/vivado/ss2_cw305_aes_pipelined.xpr) 
and [CW305 ECC](https://github.com/newaetech/chipwhisperer/blob/develop/hardware/victims/cw308_ufo_target/xc7a35/vivado/ss2_cw305_ecc.xpr) projects, ported to the CW312T-A35.

For the iCE40, there is the [CW305
AES](https://github.com/newaetech/chipwhisperer/tree/develop/hardware/victims/cw308_ufo_target/ice40up5k/ss2_aes_icestorm)
project.


### SS2 Wrapper Usage
The SS2 wrapper was designed to minimize the work required to port CW305 target
designs to target platforms which lack the CW305's SAM3U USB parallel
interface, on both the FPGA development side and the host Python side (for
CW305 designs that follow our example AES/ECC designs). 

On the Verilog side, refer to the AES and ECC examples linked above to see
what's needed. As per these examples, the wrapped CW305 top level module's
`pBYTECNT_SIZE` parameter should be set to 8, and its `pADDR_WIDTH`
parameter should be set to 32. The latter is required by `ss2.v`; the former
could in theory be changed, but
[CW305.py](../../../../software/chipwhisperer/capture/targets/CW305.py)
expects it to be 8. Stray from these recommended values at your own risk.

On the Python side, connect to an SS2 target design by adding the `platform`
argument:

`target = cw.target(scope, cw.targets.CW305, bsfile=<bitstream file>, platform='ss2')`

and everything should "just work". Calls to methods tied to CW305-specific
hardware which does not exist on the CW312T-A35 or CW312T-iCE40 (external PLL,
programmable supply voltage, USB clocking) will result in warnings.

Look at the CW305 
[AES](https://github.com/newaetech/chipwhisperer-jupyter/blob/master/demos/PA_HW_CW305_1-Attacking_AES_on_an_FPGA.ipynb)
and 
[ECC](https://github.com/newaetech/chipwhisperer-jupyter/blob/master/demos/CW305_ECC/CW305_ECC_part1.ipynb)
notebooks to see how the same Python code can interact with a CW305 target
running on either the CW305 natively, or wrapped by SS2 on the CW312T-A35 /
CW312T-iCE40.


### SS2 Wrapper Resource Overhead
The SS2 wrapper uses relatively little FPGA resources: 2% of the xc7a35's slice
LUTs and less than 1% of its slice registers.

On the much smaller iCE40, the overhead is much more significant: 20% of logic
cells. However, the [basic CW305 AES
target](https://github.com/newaetech/chipwhisperer/tree/develop/hardware/victims/cw308_ufo_target/ice40up5k/ss2_aes_icestorm)
does fit (*just barely!*).


### SS2 Wrapper Clocking
The SS2 wrapper easily meets timing at 100 MHz on Xilinx 7-series. However it
is intended to be clocked at 7.37 MHz since its baud rate (230400 bps) is
hard-coded in the Verilog source.  This can be easily modified via the
`pBIT_RATE` parameter in ss2.v, which indicates the number of clock cycles per
UART bit. Of course it's also possible to scale the clock rate and baud rate
together (e.g.  double the baud rate to 230400 * 2 = 460800 bps and double the
clock rate to 7.37e6 * 2 = 14.74e6 MHz).


### SS2 Wrapper Performance
While SimpleSerial v2.1 runs at a speedy 230400 baud, it is still
considerably slower than the CW305's SAM3U-based parallel register
interface. You can expect captures to run at about half the rate they would
on the CW305. Performance will depend on the length of the target operation
and the number and size of the target reads and writes that are required.


### SS2 Wrapper Commands
All SS2 wrapper commands use 0x23 as a command byte. The SS2 wrapper
supports 3 subcommands:

* write: `cmd=0x23, scmd=0x57, dlen=[size of data], data=[4-byte address, write data]`
* read: `cmd=0x23, scmd=0x52, dlen=5, data=[4-byte address, number of bytes to read]`
* echo: `cmd=0x23, scmd=0x45, dlen=4, data=[4-byte payload]`

The purpose of the echo command is to test that UART communication is up: it
is handled automatically by the SS2 wrapper without involving the underlying
target.

All this is tested in simulation by the [SS2 testbench](sim/test_ss2.py).

If you use
[CW305.py](../../../../software/chipwhisperer/capture/targets/CW305.py) then
you don't need to know about these details; everything is handled
under-the-hood by mapping `fpga_write()` and `fpga_read()` to the
corresponding SS2 commands.


### SS2 Next Steps
A future goal is to allow for CW305 targets which use both the parallel
SAM3U USB interface **and** UART (such as our
[DesignStartTrace](https://github.com/newaetech/DesignStartTrace)) by
expanding the SS2 wrapper to allow encapsulating UART commands inside SS2
commands.

