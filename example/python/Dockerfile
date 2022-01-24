# TIPS
# ====
# Build:
#     docker build -t voicevox_core .
# Run:
#     docker run -it voicevox_core bash

FROM python:3.9.6-slim AS build-env

# Install requirements with apt
RUN apt-get update -yqq \
&&  apt-get install -yqq \
    curl \
    unzip \
    jq

# Copy EntryPoint
COPY ./* /voicevox/

# Setup libtorch
RUN curl -sLO https://download.pytorch.org/libtorch/cu111/libtorch-cxx11-abi-shared-with-deps-1.9.0%2Bcu111.zip
RUN unzip -q libtorch*.zip && rm libtorch*.zip 
RUN mv /libtorch/lib /voicevox
RUN cd /voicevox/lib \
&&  ln -s ./libnvToolsExt-24de1d56.so.1 ./libnvToolsExt.so.1 \
&&  ln -s ./libcudart-6d56b25a.so.11.0 ./libcudart.so.11.0

# Set up built libraries
RUN curl -sLO "`curl -s https://api.github.com/repos/VOICEVOX/voicevox_core/releases/latest \
    | jq -r '.assets[]|select(.name=="core.zip")|.browser_download_url'`"
RUN unzip -q core.zip && rm core.zip
RUN mv core/* /voicevox/

FROM python:3.9.6-slim

# copy mudules
COPY --from=build-env /voicevox /voicevox
WORKDIR /voicevox

# Add libtorch to LD_LIBRARY_PATH
ENV LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/voicevox/lib/"

# Install requirements with apt
RUN apt-get update -yqq \
&&  apt-get install -yqq \
    cmake \
    libsndfile-dev \
    git \
    g++ \
&&  apt-get clean \
&&  rm -rf /var/lib/apt/lists/*

# Install requirements with pip
RUN pip install -U pip && pip install -q -r requirements.txt

# Install pyopenjtalk/dic.tar.gz for openjtalk from sourceforge
RUN python -c 'import pyopenjtalk;pyopenjtalk._lazy_init()'

# Install voicevox_core
RUN LIBRARY_PATH="$LIBRARY_PATH:." python setup.py install

# CMD bash
