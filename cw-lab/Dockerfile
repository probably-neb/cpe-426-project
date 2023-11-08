FROM ubuntu:20.04

RUN sudo apt get update && sudo apt get upgrade -y

RUN sudo apt install git python3 python3-pip python3-tk avr-libc gcc-avr ggc-arm-none-eabi

RUN git clone https://github.com/newaetech/chipwhisperer-jupyter.git

CMD bash
