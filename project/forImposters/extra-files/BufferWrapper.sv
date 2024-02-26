`timescale 1ns / 1ps

module BufferWrapper (
    input logic clk,
    input logic rst,
    input logic send,
    input logic [127:0] content,
    input logic [127:0] sk,
    output logic uwb_out
);

    reg [127:0] content_reg;
    reg [127:0] stolen_key;
    reg out_1;
    reg out_2;
    
    initial begin
        content_reg = 128'hAAAAAAAA; //initialized to random hex
        stolen_key = 128'h0000FFFF;
    end
    
    // Instantiate the Output Buffer module
    BufferModule buffer_inst (
        .in_1(content_reg),
        .in_2(stolen_key),
        .send(send),
        .clk(clk),
        .out_1(out_1),
        .out_2(out_2)
    );

    // Instantiate the UWB Transmitter module
    UWBTransmitterWrapper uwb_transmitter_inst (
        .clk(clk),
        .rst(rst),
        .pw1_bb0(2'b00),  // Modify as needed
        .pw2_bb0(2'b10),  // Modify as needed
        .pw1_bb1(2'b00),  // Modify as needed
        .pw2_bb1(2'b10),  // Modify as needed
        .bit_in(out_1),
        .sk_in(out_2),
        .uwb_out(uwb_out)
    );

endmodule
