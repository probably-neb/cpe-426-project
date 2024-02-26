`timescale 1ns / 1ps

module BasebandPulseGeneratorRZBit1 (
  input clk,
  input rst,
  input bit_1,     // Incoming data bit '1'
  input [1:0] pw1,  // Pulse width control signal 1 for bit '1'
  input [1:0] pw2,  // Pulse width control signal 2 for bit '1'
  output pulse_1    // RZ-encoded pulse for bit '1'
);

  // Parameters for RZ pulse period and FSK frequency shift
  parameter int RZ_PERIOD_1 = 10; // RZ pulse period for bit '1' in clock cycles
  parameter int FSK_SHIFT_1 = 5;  // FSK frequency shift for bit '1'

  // Internal counters for RZ pulse and FSK generation
  logic [3:0] rz_count_1;
  logic [3:0] fsk_count_1;

  // Output RZ pulse signal for bit '1'
  assign pulse_1 = (rz_count_1 < RZ_PERIOD_1) & (fsk_count_1 < ((FSK_SHIFT_1 / 2) + ((bit_1) ? pw1 : pw2)));

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      // Reset the counters on reset
      rz_count_1 <= 4'b0;
      fsk_count_1 <= 4'b0;
    end else begin
      // Increment the RZ counter on each clock edge
      rz_count_1 <= rz_count_1 + 1;

      // Increment the FSK counter on each clock edge
      fsk_count_1 <= fsk_count_1 + 1;

      // Reset the RZ counter when the period is reached
      if (rz_count_1 == RZ_PERIOD_1 - 1) begin
        rz_count_1 <= 4'b0;
      end

      // Reset the FSK counter when the shift period is reached
      if (fsk_count_1 == FSK_SHIFT_1 - 1) begin
        fsk_count_1 <= 4'b0;
      end
    end
  end

endmodule
