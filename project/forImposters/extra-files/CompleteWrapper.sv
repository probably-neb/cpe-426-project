`timescale 1ns / 1ps

module CompleteWrapper (
    input logic CLK,
    input logic RST,
    input logic SEND,
    output logic UWB_OUT,
    output logic AMP_OUT
);

    // Instantiate BufferWrapper
    BufferWrapper buffer_wrapper_inst (
        .clk(CLK),
        .rst(RST),
        .send(SEND),
        .uwb_out(UWB_OUT)
    );

    // Instantiate PowerAmplifier
    PowerAmplifier power_amplifier_inst (
        .amp_in(UWB_OUT),   // Connect UWB_OUT to PowerAmplifier's amp_in
        .amp_out(AMP_OUT)
    );

endmodule
