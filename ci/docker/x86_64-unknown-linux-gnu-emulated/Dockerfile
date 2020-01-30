FROM ubuntu:18.04
RUN apt-get update && apt-get install -y --no-install-recommends \
  gcc \
  libc6-dev \
  file \
  make \
  ca-certificates \
  wget \
  bzip2 \
  cmake \
  libclang-dev \
  clang

RUN wget https://github.com/gnzlbg/intel_sde/raw/1d57a96dcfe8e7ad3f03081d0c1d965580ed530b/sde-external-8.35.0-2019-03-11-lin.tar.bz2
RUN tar -xjf sde-external-8.35.0-2019-03-11-lin.tar.bz2
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="/sde-external-8.35.0-2019-03-11-lin/sde64 --"
