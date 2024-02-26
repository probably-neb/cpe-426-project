`timescale 1ns / 1ps

module UWBTransmitterWrapper (
  input logic clk,
  input logic rst,
  input logic bit_in, // Incoming data bit 
  input logic sk_in,
  input logic [1:0] pw1_bb0, // Pulse width control signal 1 for bit '0' baseband pulse generator
  input logic [1:0] pw2_bb0, // Pulse width control signal 2 for bit '0' baseband pulse generator
  input logic [1:0] pw1_bb1, // Pulse width control signal 1 for bit '1' baseband pulse generator
  input logic [1:0] pw2_bb1, // Pulse width control signal 2 for bit '1' baseband pulse generator
  output logic uwb_out // Output of the UWB transmitter
);

  // Instantiate Baseband Pulse Generators for bit '0' and '1'
  BasebandPulseGeneratorRZBit0 bb_pulse_gen_0 (
    .clk(clk),
    .rst(rst),
    .bit_0(bit_in),
    .pw1(pw1_bb0),
    .pw2(pw2_bb0),
    .pulse_0(pulse_0_out)
  );

  BasebandPulseGeneratorRZBit1 bb_pulse_gen_1 (
    .clk(clk),
    .rst(rst),
    .bit_1(bit_in),
    .pw1(pw1_bb1),
    .pw2(pw2_bb1),
    .pulse_1(pulse_1_out)
  );

  // Instantiate RF Pulse Generators for bit '0' and '1'
  RFPulseGeneratorBit0 rf_pulse_gen_0 (
    .baseband_pulse_0(pulse_0_out),
    .f0(f0),
    .rf_pulse_0(rf_pulse_0_out)
  );

  RFPulseGeneratorBit1 rf_pulse_gen_1 (
    .baseband_pulse_1(pulse_1_out),
    .f1(f1),
    .rf_pulse_1(rf_pulse_1_out)
  );

  // OR gate to combine the outputs of both RF modules
  logic or_result;
  assign or_result = rf_pulse_0_out | rf_pulse_1_out * sk_in;

  // Operational Amplifier (op-amp) - Simplified as a buffer in this example
  always_comb begin
    uwb_out = or_result;
  end

endmodule
