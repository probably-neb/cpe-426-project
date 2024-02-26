`timescale 1ns / 1ps

module PowerAmplifier (
    input logic amp_in,
    output logic amp_out
);

    // Parameter
    parameter int GAIN = 2; // Gain of the amplifier
    parameter int FIXED_POINT_BITS = 8; // Adjust based on your desired precision

    // Internal signals
    logic [2*FIXED_POINT_BITS-1:0] amplified_signal;
    
    // Class-AB Power Amplifier behavior
    always_comb begin
        amplified_signal = GAIN * { {FIXED_POINT_BITS{1'b0}}, amp_in };
    end

    // Output
    assign amp_out = (amplified_signal > (1 << (FIXED_POINT_BITS - 1))) ? 1'b1 : 1'b0;

endmodule
