`timescale 1ns / 1ps

module BasebandPulseGeneratorRZBit0 (
  input clk,
  input rst,
  input bit_0,     // Incoming data bit '0'
  input [1:0] pw1,  // Pulse width control signal 1 for bit '0'
  input [1:0] pw2,  // Pulse width control signal 2 for bit '0'
  output pulse_0    // RZ-encoded pulse for bit '0'
);

  // Parameters for RZ pulse period and FSK frequency shift
  parameter int RZ_PERIOD_0 = 20; // RZ pulse period for bit '0' in clock cycles
  parameter int FSK_SHIFT_0 = 5;  // FSK frequency shift for bit '0'

  // Internal counters for RZ pulse and FSK generation
  logic [3:0] rz_count_0;
  logic [3:0] fsk_count_0;

  // Output RZ pulse signal for bit '0'
  assign pulse_0 = (rz_count_0 < RZ_PERIOD_0) & (fsk_count_0 < ((FSK_SHIFT_0 / 2) + ((bit_0) ? pw1 : pw2)));

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      // Reset the counters on reset
      rz_count_0 <= 4'b0;
      fsk_count_0 <= 4'b0;
    end else begin
      // Increment the RZ counter on each clock edge
      rz_count_0 <= rz_count_0 + 1;

      // Increment the FSK counter on each clock edge
      fsk_count_0 <= fsk_count_0 + 1;

      // Reset the RZ counter when the period is reached
      if (rz_count_0 == RZ_PERIOD_0 - 1) begin
        rz_count_0 <= 4'b0;
      end

      // Reset the FSK counter when the shift period is reached
      if (fsk_count_0 == FSK_SHIFT_0 - 1) begin
        fsk_count_0 <= 4'b0;
      end
    end
  end

endmodule
