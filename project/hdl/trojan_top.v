`timescale 1ns / 1ps

module trojan_top(
    input clk_100MHz,       // basys 3 FPGA clock signal
    input reset,            // btnC    
    input rx,               // USB-RS232 Rx
    output tx,              // USB-RS232 Tx
    output [3:0] an,        // 7 segment display digits
    output [0:6] seg,       // 7 segment display segments
    output [7:0] LED,        // data byte display
    output key_led
    );
    
    // Connection Signals
    wire rx_full, rx_empty;
    wire aes_done;
    wire [7:0] rec_data, rec_data1;
    reg send;
    wire [128-1:0] read_mem_wire;
    reg got_key;
    reg [128-1:0] key;
    wire [128-1:0] aes_out;
    reg start_reg, load_reg;
    reg [63:0] key_half;
    reg [63:0] data_half;
    reg [2:0] aes_state;
    reg aes_reset;
    reg waiting;
    
    // Complete UART Core
    uart_top UART_UNIT
        (
            .clk_100MHz(clk_100MHz),
            .reset(reset),
            .read_uart(send),
            .write_uart(send),
            .rx(rx),
            .write_data(rec_data1),
            .rx_full(rx_full),
            .rx_empty(rx_empty),
            .read_data(rec_data),
            .tx(tx),
            .read_mem_wire(read_mem_wire),
            .write_all(1'b1),
            .write_mem_wire(aes_out)
        );

        
    aes128_fast AES (
        .clk(clk_100MHz),
        .reset(combined_reset),
        .start(start_reg),
        .mode(1'b1),
        .load(load_reg),
        .key(key_half),
        .data_in(data_half),
        .data_out(aes_out),
        .done(aes_done)
    );

    // Signal Logic    
    assign rec_data1 = rec_data;    // add 1 to ascii value of received data (to transmit)
    
    // Output Logic
    assign LED = rec_data;              // data byte received displayed on LEDs
    assign an = 4'b1110;                // using only one 7 segment digit 
    assign seg = {~rx_full, 2'b11, ~rx_empty, 3'b111};
    assign key_led = got_key;
    
    assign combined_reset = aes_reset | reset;
            
    always @ (posedge rx_full or posedge reset) begin
        if (reset) begin
            key <= 128'b0;
            got_key <= 1'b0;
        end else if (~got_key) begin
            key <= read_mem_wire;
            got_key <= 1'b1;
        end
    end

    always @ (posedge clk_100MHz) begin
        if (reset) begin
            aes_state <= 3'b000;
            send <= 1'b0;
            key_half <= 64'b0;
            data_half <= 64'b0;
            start_reg <= 1'b0;
            load_reg <= 1'b0;
            waiting <= 1'b0;
        end else if (rx_full && got_key) begin
            case (aes_state)
                3'b000: begin
                    load_reg <= 1'b1;
                    key_half <= key[127:64];
                    data_half <= read_mem_wire[127:64];
                    aes_state <= 3'b001;
                end
                
                3'b001: begin
                    load_reg <= 1'b0;
                    key_half <= key[63:0];
                    data_half <= read_mem_wire[63:0];
                    aes_state <= 3'b010;
                end
                
                3'b010: begin
                    aes_state <= 3'b011;
                end
                
                3'b011: begin
                    start_reg <= 1'b1;
                    aes_state <= 3'b100;
                end
                
                3'b100: begin
                    start_reg <= 1'b0;
                    aes_state <= 3'b101;
                end

                3'b101: begin
                    if (~waiting) begin
                        aes_reset <= 1;
                        aes_state <= 3'b110;
                    end
                end
                
                3'b110: begin
                    aes_reset <= 0;
                    aes_state <= 3'b000;
                    waiting <= 1'b1;
                end
            endcase
        end
        
        if (waiting && aes_done && ~rx_empty) begin
            send <= 1'b1;
        end else if (waiting && aes_done && rx_empty) begin
            send <= 1'b0;
            waiting <= 1'b0;
        end
    end
endmodule
