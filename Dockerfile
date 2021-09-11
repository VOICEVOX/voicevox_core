FROM python:3.8

RUN apt-get update

# install requirements with apt
RUN apt-get install curl git unzip jq -yqq

# setup libtorch
RUN curl -LO https://download.pytorch.org/libtorch/nightly/cpu/libtorch-shared-with-deps-latest.zip \
    && unzip libtorch-shared-with-deps-latest.zip

# add libtorch to LD_LIBRARY_PATH
RUN export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/libtorch/lib/"
RUN echo LD_LIBRARY_PATH=`echo $LD_LIBRARY_PATH|tr : \\n`

# clone repo
RUN git clone --depth 1 https://github.com/Hiroshiba/voicevox_core

# cd
WORKDIR voicevox_core/example/python

# set up built libraries
RUN curl -LO "`curl -s https://api.github.com/repos/Hiroshiba/voicevox_core/releases/latest \
    | jq -r '.assets[]|select(.name=="core.zip")|.browser_download_url'`" \
    && unzip core.zip

# install requirements with pip
RUN pip install -U pip && pip install -r requirements.txt

# install voicevox_core
RUN python setup.py install
CMD ["bash", "-c"]
