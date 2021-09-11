# TIPS
# ====
# Build:
#     docker build -t voicevox_core .
# Run:
#     docker run -it voicevox_core bash

FROM python:3.9.6

RUN apt-get update -yqq

# Install requirements with apt
RUN apt-get install -yqq \
    curl cmake git \
    unzip jq libsndfile-dev

# Setup libtorch
RUN curl -sLO https://download.pytorch.org/libtorch/cu111/libtorch-cxx11-abi-shared-with-deps-1.9.0%2Bcu111.zip
RUN unzip -q libtorch*.zip && rm libtorch*.zip
RUN cp /libtorch/lib/libnvToolsExt-24de1d56.so.1 /libtorch/lib/libnvToolsExt.so.1
RUN cp /libtorch/lib/libcudart-6d56b25a.so.11.0 /libtorch/lib/libcudart.so.11.0

# Add libtorch to LD_LIBRARY_PATH
ENV LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/libtorch/lib/"

# Clone repo
RUN git clone -q --depth 1 https://github.com/Hiroshiba/voicevox_core

# Change working dir to voicevox_core_example/python
WORKDIR voicevox_core/example/python

# Set up built libraries
RUN curl -sLO "`curl -s https://api.github.com/repos/Hiroshiba/voicevox_core/releases/latest \
    | jq -r '.assets[]|select(.name=="core.zip")|.browser_download_url'`"
RUN unzip -q core.zip && rm core.zip

RUN mv core/* .

# Install requirements with pip
RUN pip install -U pip && pip install -q -r requirements.txt

# Install voicevox_core
RUN LIBRARY_PATH="$LIBRARY_PATH:." python setup.py install

# CMD bash
