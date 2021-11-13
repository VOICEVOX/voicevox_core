# syntax=docker/dockerfile:1.3-labs
ARG BASE_IMAGE=ubuntu:bionic
FROM ${BASE_IMAGE}

ARG CC_VERSION=8
ARG CXX_VERSION=8

ARG ARCH=arm-linux-gnueabihf

ENV CC=${ARCH}-gcc-${CC_VERSION}
ENV CXX=${ARCH}-g++-${CXX_VERSION}

RUN <<EOF
    apt-get update
    apt-get install -y \
        build-essential \
        git \
        wget \
        gcc-${CC_VERSION}-${ARCH} \
        g++-${CXX_VERSION}-${ARCH} \
        python3
EOF

# ONNX Runtime v1.9.0 requires CMake 3.18 or higher.
ARG CMAKE_VERSION=3.22.0-rc2
RUN <<EOF
    wget -O /tmp/cmake.sh https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-linux-x86_64.sh
    bash /tmp/cmake.sh --skip-license --prefix=/usr/local
EOF

ARG ONNXRUNTIME_VERSION=v1.8.2
RUN <<EOF
    git clone --recursive https://github.com/microsoft/onnxruntime.git
    cd onnxruntime

    git checkout "${ONNXRUNTIME_VERSION}"

    git submodule sync
    git submodule update --init --recursive --jobs 0
EOF

# add --arm for gcc-8: https://github.com/microsoft/onnxruntime/issues/4189
# skip test: https://github.com/microsoft/onnxruntime/issues/2436
# CMAKE_SYSTEM_PROCESSOR: https://github.com/microsoft/onnxruntime/releases/tag/v1.9.0
ARG ATOMIC=1
ARG CMAKE_SYSTEM_PROCESSOR=armv7l
RUN <<EOF
    cd onnxruntime

    if [ "${ATOMIC}" = "1" ]; then
        echo 'string(APPEND CMAKE_C_FLAGS " -latomic")' >> cmake/CMakeLists.txt
        echo 'string(APPEND CMAKE_CXX_FLAGS " -latomic")' >> cmake/CMakeLists.txt
        echo "set(CMAKE_SYSTEM_PROCESSOR \"${CMAKE_SYSTEM_PROCESSOR}\")" >> cmake/CMakeLists.txt
    fi
EOF

ARG LD_SYMLINK_NAME=ld-linux-armhf.so.3
RUN <<EOF
    if [ -n "${LD_SYMLINK_NAME}" ]; then
        ln -s /usr/${ARCH}/lib /lib/${ARCH}
        ln -s /lib/${ARCH}/ld-*.so /lib/${LD_SYMLINK_NAME}
    fi
EOF

WORKDIR /onnxruntime
