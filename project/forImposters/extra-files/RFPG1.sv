`timescale 1ns / 1ps

module RFPulseGeneratorBit1 (
  input f1, // Frequency control input for bit '1'
  input baseband_pulse_1, // Output of the earlier baseband pulse generator for bit '1'
  output rf_pulse_1 // RF pulse for bit '1'
);

  // Parameters for RF pulse period
  parameter int RF_PERIOD_MIN = 50; // Minimum RF pulse period for bit '1' in clock cycles
  parameter int RF_PERIOD_MAX = 150; // Maximum RF pulse period for bit '1' in clock cycles

  // Internal counter and period for RF pulse generation
  logic [6:0] rf_count_1;
  logic [6:0] rf_period_1;

  // Output RF pulse signal for bit '1'
  assign rf_pulse_1 = (rf_count_1 < rf_period_1) & baseband_pulse_1;

  always_ff @(posedge baseband_pulse_1) begin
    // Adjust RF pulse period based on f1
    rf_period_1 <= RF_PERIOD_MIN + f1;

    // Increment the RF counter on each rising edge of baseband_pulse_1
    rf_count_1 <= rf_count_1 + 1;

    // Reset the RF counter when the period is reached
    if (rf_count_1 == rf_period_1 - 1) begin
      rf_count_1 <= 7'b0;
    end
  end

endmodule
