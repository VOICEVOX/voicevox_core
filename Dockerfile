FROM python:3.8

RUN apt-get update
RUN apt-get install curl git unzip jq -yqq
RUN curl -LO https://download.pytorch.org/libtorch/nightly/cpu/libtorch-shared-with-deps-latest.zip \
    && unzip libtorch-shared-with-deps-latest.zip
RUN export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/libtorch/lib/"
RUN echo LD_LIBRARY_PATH=`echo $LD_LIBRARY_PATH|tr : \\n`

RUN git clone --depth 1 https://github.com/Hiroshiba/voicevox_core
WORKDIR voicevox_core/example/python
RUN curl -LO "`curl -s https://api.github.com/repos/Hiroshiba/voicevox_core/releases/latest \
    | jq -r '.assets[]|select(.name=="core.zip")|.browser_download_url'`" \
    && unzip core.zip
RUN pip install -U pip && pip install -r requirements.txt
RUN python setup.py install
CMD ["bash", "-c"]
