`timescale 1ns / 1ps

module BufferModule (
    input wire [127:0] in_1,
    input wire [127:0] in_2,
    input wire send,
    input wire clk,
    output reg out_1,
    output reg out_2
);

    reg [127:0] buffer_1;
    reg [127:0] buffer_2;
    reg [6:0] current_bit; // Counter to track the current bit position

    always @(posedge clk) begin
        if (send) begin
            buffer_1 <= in_1;
            buffer_2 <= in_2;
            current_bit <= 0;
        end else if (current_bit < 128) begin
            out_1 <= buffer_1[current_bit];
            out_2 <= buffer_2[current_bit];
            current_bit <= current_bit + 1;
        end
    end

endmodule