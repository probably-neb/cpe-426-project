/* 
ChipWhisperer Artix Target - Simple testbench to check for signs of life.

Copyright (c) 2020, NewAE Technology Inc.
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted without restriction. Note that modules within
the project may have additional restrictions, please carefully inspect
additional licenses.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

The views and conclusions contained in the software and documentation are those
of the authors and should not be interpreted as representing official policies,
either expressed or implied, of NewAE Technology Inc.
*/

`timescale 1ns / 1ns
`default_nettype none 

`include "cw305_pmul_defines.v"

module tb();
    parameter pADDR_WIDTH = 21;
    parameter pBYTECNT_SIZE = 8;
    parameter pUSB_CLOCK_PERIOD = 10;
    parameter pPLL_CLOCK_PERIOD = 6;
    parameter pSEED = 1;
    parameter pTIMEOUT = 30000;
    parameter pVERBOSE = 0;
    parameter pDUMP = 0;

    reg usb_clk;
    reg usb_clk_enable;
    wire [7:0] usb_data;
    reg [7:0] usb_wdata;
    reg [pADDR_WIDTH-1:0] usb_addr;
    reg usb_rdn;
    reg usb_wrn;
    reg usb_cen;
    reg usb_trigger;

    reg j16_sel;
    reg k16_sel;
    reg k15_sel;
    reg l14_sel;
    reg pushbutton;
    reg pll_clk1;
    wire tio_clkin;
    wire trig_out;

    wire led1;
    wire led2;
    wire led3;

    wire tio_trigger;
    wire tio_clkout;


    int seed;
    int errors;
    int warnings;
    int i;
    
    reg [31:0] write_data;

    wire clk = pll_clk1;  // shorthand for testbench

   int cycle;
   int total_time;

   reg [255:0] read_data;
   reg [255:0] k = 256'h70a12c2db16845ed56ff68cfc21a472b3f04d7d6851bf6349f2d7d5b3452b38a;
   //reg [255:0] k = 256'h000f0000ffff0000ffffffff000000003f04d7d6851bf6349f2d7d5b3452b38a;
   reg [255:0] gx = 256'h6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296;
   reg [255:0] gy = 256'h4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5;
   reg [255:0] expected_rx = 256'h8101ece47464a6ead70cf69a6e2bd3d88691a3262d22cba4f7635eaff26680a8;
   reg [255:0] expected_ry = 256'hd8a12ba61d599235f67d9cb4d58f1783d3ca43e78f0a5abaa624079936c0c3a9;
	

   initial begin
      seed = pSEED;
      errors = 0;
      warnings = 0;
      $display("Running with seed=%0d", seed);
      //$urandom(seed);
      if (pDUMP) begin
         $dumpfile("results/tb.fst");
         $dumpvars(0, tb);
      end
      usb_clk = 1'b1;
      usb_clk_enable = 1'b1;
      pll_clk1 = 1'b1;

      usb_wdata = 0;
      usb_addr = 0;
      usb_rdn = 1;
      usb_wrn = 1;
      usb_cen = 1;
      usb_trigger = 0;

      j16_sel = 0;
      k16_sel = 0;
      k15_sel = 0;
      l14_sel = 0;
      pushbutton = 1;
      pll_clk1 = 0;

      #(pUSB_CLOCK_PERIOD*2) pushbutton = 0;
      #(pUSB_CLOCK_PERIOD*2) pushbutton = 1;
      #(pUSB_CLOCK_PERIOD*10);

      write_bytes(0, 32, `REG_CRYPT_K, k);
      write_bytes(0, 32, `REG_CRYPT_GX, gx);
      write_bytes(0, 32, `REG_CRYPT_GY, gy);

      $display("Encrypting via register...");
      write_byte(0, `REG_CRYPT_GO, 0, 1);
      repeat (5) @(posedge usb_clk);
      wait_done();

      read_bytes(0, 32, `REG_CRYPT_RX, read_data);
      if (read_data !== expected_rx) begin
         errors += 1;
         $display("ERROR on rx: expected %h", expected_rx);
         $display("                  got %h", read_data);
      end
      else
         $display("Good rx: %h", read_data);

      read_bytes(0, 32, `REG_CRYPT_RY, read_data);
      if (read_data !== expected_ry) begin
         errors += 1;
         $display("ERROR on ry: expected %h", expected_ry);
         $display("                  got %h", read_data);
      end
      else
         $display("Good ry: %h", read_data);



      /*
      $display("Encrypting via usb_trigger (USB clock disabled)...");
      write_bytes(0, 1, `REG_CRYPT_TEXTIN, 8'h01);
      expected_cipher = 128'h0efee0bff4cf170752994fb45bd45934;
      usb_clk_enable = 1'b0;
      @(posedge usb_clk) usb_trigger = 1'b1;
      repeat (10) @(posedge usb_clk); 
      usb_trigger = 1'b0;
      repeat (30) @(posedge pll_clk1);
      usb_clk_enable = 1'b1;
      repeat (5) @(posedge usb_clk);
      wait_done();
      read_bytes(0, 16, `REG_CRYPT_CIPHEROUT, read_data);
      if (read_data == expected_cipher) begin
         $display("Good result");
      end
      else begin
         errors += 1;
         $display("ERROR: expected %h", expected_cipher);
         $display("            got %h", read_data);
      end


      $display("Encrypting via usb_trigger (USB clock enabled)...");
      write_bytes(0, 1, `REG_CRYPT_TEXTIN, 8'h02);
      expected_cipher = 128'h8623e205b50bede46f795d1aad15faae;
      @(posedge usb_clk) usb_trigger = 1'b1;
      repeat (10) @(posedge usb_clk); 
      usb_trigger = 1'b0;
      repeat (30) @(posedge pll_clk1);
      repeat (5) @(posedge usb_clk);
      wait_done();
      read_bytes(0, 16, `REG_CRYPT_CIPHEROUT, read_data);
      if (read_data == expected_cipher) begin
         $display("Good result");
      end
      else begin
         errors += 1;
         $display("ERROR: expected %h", expected_cipher);
         $display("            got %h", read_data);
      end

      $display("Encrypting via usb_trigger (USB clock disabled, returns mid-encryption)...");
      write_bytes(0, 1, `REG_CRYPT_TEXTIN, 8'h03);
      expected_cipher = 128'h46be87df4d18bfe6d0d1d3b20b6bf382;
      usb_clk_enable = 1'b0;
      @(posedge usb_clk) usb_trigger = 1'b1;
      repeat (2) @(posedge usb_clk); 
      usb_trigger = 1'b0;
      repeat (2) @(posedge pll_clk1);
      usb_clk_enable = 1'b1;
      repeat (1) @(posedge usb_clk);
      wait_done();
      read_bytes(0, 16, `REG_CRYPT_CIPHEROUT, read_data);
      if (read_data == expected_cipher) begin
         $display("Good result");
      end
      else begin
         errors += 1;
         $display("ERROR: expected %h", expected_cipher);
         $display("            got %h", read_data);
      end
      */



      $display("done!");
      #(pUSB_CLOCK_PERIOD*10);
      if (errors)
         $display("SIMULATION FAILED (%0d errors, %0d warnings).", errors, warnings);
      else
         $display("Simulation passed (%0d warnings).", warnings);
      $finish;

   end

   // maintain a cycle counter
   always @(posedge clk) begin
      if (pushbutton == 0)
         cycle <= 0;
      else
         cycle <= cycle + 1;
   end


   // timeout thread:
   initial begin
      #(pUSB_CLOCK_PERIOD*pTIMEOUT);
      errors += 1;
      $display("ERROR: global timeout");
      $display("SIMULATION FAILED (%0d errors).", errors);
      $finish;
   end


   reg read_select;

   assign usb_data = read_select? 8'bz : usb_wdata;
   assign tio_clkin = pll_clk1;

   always @(*) begin
      if (usb_wrn == 1'b0)
         read_select = 1'b0;
      else if (usb_rdn == 1'b0)
         read_select = 1'b1;
   end

   `include "tb_cw305_reg_tasks.v"

   always #(pUSB_CLOCK_PERIOD/2) usb_clk = !usb_clk;
   always #(pPLL_CLOCK_PERIOD/2) pll_clk1 = !pll_clk1;

   wire #1 usb_rdn_out = usb_rdn;
   wire #1 usb_wrn_out = usb_wrn;
   wire #1 usb_cen_out = usb_cen;
   wire #1 usb_trigger_out = usb_trigger;

   wire trigger; // TODO: use it?

   reg [31:0] cycle_count;
   always @(posedge pll_clk1) begin
      if (U_dut.crypt_start) begin
         $display("Pmul started");
         cycle_count <= 0;
      end
      else begin
         cycle_count <= cycle_count + 1;
         if ((cycle_count > 0) && ((cycle_count % 1e5) == 0))
            $display("Heartbeat: cycle = %d", cycle_count);
      end
   end

   cw305_ecc_p256_pmul_top #(
      .pBYTECNT_SIZE            (pBYTECNT_SIZE),
      .pADDR_WIDTH              (pADDR_WIDTH)
   ) U_dut (
      .usb_clk                  (usb_clk & usb_clk_enable),
      .usb_data                 (usb_data),
      .usb_addr                 (usb_addr),
      .usb_rdn                  (usb_rdn_out),
      .usb_wrn                  (usb_wrn_out),
      .usb_cen                  (usb_cen_out),
      .usb_trigger              (usb_trigger_out),
      .j16_sel                  (j16_sel),
      .k16_sel                  (k16_sel),
      .k15_sel                  (k15_sel),
      .l14_sel                  (l14_sel),
      .pushbutton               (pushbutton),
      .led1                     (led1),
      .led2                     (led2),
      .led3                     (led3),
      .pll_clk1                 (pll_clk1),
      .tio_trigger              (trigger),
      .tio_clkout               (),             // unused
      .tio_clkin                (tio_clkin)
   );


   task wait_done;
      bit busy;
      busy = 1;
      while (busy == 1) begin
         //$display("checking busy...");
         read_byte(0, `REG_CRYPT_GO, 0, busy);
      end
   endtask


endmodule

`default_nettype wire

