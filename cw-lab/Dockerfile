FROM ubuntu:20.04

SHELL ["/bin/bash", "-c"]

RUN apt update && apt upgrade

ENV DEBIAN_FRONTEND=noninteractive
# python prereqs
RUN apt-get -y install build-essential gdb lcov pkg-config \
        libbz2-dev libffi-dev libgdbm-dev libgdbm-compat-dev liblzma-dev \
        libncurses5-dev libreadline6-dev libsqlite3-dev libssl-dev \
        lzma lzma-dev tk-dev uuid-dev zlib1g-dev curl
RUN apt install -y libusb-dev make git avr-libc gcc-avr \
        gcc-arm-none-eabi libusb-1.0-0-dev usbutils
# install pyenv - skip if already done
# RUN curl https://pyenv.run | bash
# RUN echo 'export PATH="~/.pyenv/bin:$PATH"' >> ~/.bashrc
# RUN echo 'export PATH="~/.pyenv/shims:$PATH"' >> ~/.bashrc
# RUN echo 'eval "$(pyenv init -)"' >> ~/.bashrc
# RUN echo 'eval "$(pyenv virtualenv-init -)"' >> ~/.bashrc
# RUN source ~/.bashrc
# RUN pyenv install 3.9.5
# RUN pyenv virtualenv 3.9.5 cw
# RUN pyenv activate cw
# RUN groupadd -r imposter -g 433 && \
    # useradd -u 431 -r -g imposter -s /sbin/nologin -c "Docker image user" imposter
RUN apt-get update \
 && apt-get install -y sudo
# RUN adduser --disabled-password --gecos '' imposter
# RUN adduser imposter sudo
# RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

# RUN mkhomedir_helper imposter
# USER imposter
RUN mkdir /home/imposters
WORKDIR /home/imposters

# clone repo as imposter user
RUN git clone --depth=1 https://github.com/newaetech/chipwhisperer

# back to root to setup permissions
# USER root
RUN apt-get install -y udev
RUN mkdir -p /etc/udev/rules.d
RUN cp /home/imposters/chipwhisperer/hardware/50-newae.rules /etc/udev/rules.d/50-newae.rules
RUN /lib/systemd/systemd-udevd --daemon && udevadm control --reload-rules
# RUN groupadd -f chipwhisperer
# RUN usermod -aG chipwhisperer imposter
# RUN usermod -aG plugdev imposter

# install python
RUN apt-get install -y python3 python3-pip

WORKDIR /home/imposters/chipwhisperer

RUN git config --global --add safe.directory /home/imposter/chipwhisperer
RUN git submodule update --init jupyter
# USER root
# ENV PATH="${PATH}:/home/imposters/.local/bin"
RUN python3 -m pip install -e .
RUN echo "traitlets<5.10" >> jupyter/requirements.txt
RUN python3 -m pip install -r jupyter/requirements.txt


CMD bash
