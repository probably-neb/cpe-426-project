`timescale 1ns / 1ps

module RFPulseGeneratorBit0 (
  input f0, // Frequency control input for bit '0'
  input baseband_pulse_0, // Output of the earlier baseband pulse generator for bit '0'
  output rf_pulse_0 // RF pulse for bit '0'
);

  // Parameters for RF pulse period
  parameter int RF_PERIOD_MIN = 100; // Minimum RF pulse period for bit '0' in clock cycles
  parameter int RF_PERIOD_MAX = 200; // Maximum RF pulse period for bit '0' in clock cycles

  // Internal counter and period for RF pulse generation
  logic [6:0] rf_count_0;
  logic [6:0] rf_period_0;

  // Output RF pulse signal for bit '0'
  assign rf_pulse_0 = (rf_count_0 < rf_period_0) & baseband_pulse_0;

  always_ff @(posedge baseband_pulse_0) begin
    // Adjust RF pulse period based on f0
    rf_period_0 <= RF_PERIOD_MIN + f0;

    // Increment the RF counter on each rising edge of baseband_pulse_0
    rf_count_0 <= rf_count_0 + 1;

    // Reset the RF counter when the period is reached
    if (rf_count_0 == rf_period_0 - 1) begin
      rf_count_0 <= 7'b0;
    end
  end

endmodule
