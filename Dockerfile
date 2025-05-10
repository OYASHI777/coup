# Use minimal rust base image
FROM rust:1.75

RUN apt-get update && apt-get install -y \
  git \
  && rm -rf /var/lib/apt/lists/*

# Create non-root user
ARG USER=devuser
ARG UID=1337
RUN useradd -m -u ${UID} ${USER}

# Uncomment to switch to non-root user
# USER ${USER}

# set working directory in container where proj is mounted
WORKDIR /home/${USER}/coup

# Starts interactive shell
CMD ["bash"]
