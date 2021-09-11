# Build:
#     docker build -t voicevox_core .
# Run:
#     docker run -it voicevox_core

FROM python:3.9.6

RUN apt-get update -yqq

# install requirements with apt
RUN apt-get install -yqq curl cmake git unzip jq

# setup libtorch
RUN curl -sLO https://download.pytorch.org/libtorch/cu111/libtorch-cxx11-abi-shared-with-deps-1.9.0%2Bcu111.zip \
    && unzip -q "libtorch-shared-with-deps-1.9.0+cu111.zip"

# add libtorch to LD_LIBRARY_PATH
ENV LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/libtorch/lib/"

# clone repo
RUN git clone -q --depth 1 https://github.com/Hiroshiba/voicevox_core

# cd
WORKDIR voicevox_core/example/python

# set up built libraries
RUN curl -sLO "`curl -s https://api.github.com/repos/Hiroshiba/voicevox_core/releases/latest \
    | jq -r '.assets[]|select(.name=="core.zip")|.browser_download_url'`" \
    && unzip -q core.zip

RUN mv core/* .

# install requirements with pip
RUN pip install -U pip && pip install -q -r requirements.txt

# install voicevox_core
RUN LIBRARY_PATH="$LIBRARY_PATH:." python setup.py install
CMD bash
