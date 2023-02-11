# install system-wide dependencies
apt-get -yq update && \
    apt-get -yq install curl libsdl2-dev libssl-dev openssh-client build-essential

# install rust
export PATH="/root/.cargo/bin:${PATH}"

# install conda and setup python 3.8 environment
curl -L -O "https://github.com/conda-forge/miniforge/releases/latest/download/Mambaforge-$(uname)-$(uname -m).sh"
bash Mambaforge-$(uname)-$(uname -m).sh -b -p /opt/conda
export PATH=/opt/conda/bin:$PATH
conda create -n dev python=3.8

# install pytorch
/bin/bash -c ". activate dev && pip install torch===1.11.0"

# configure env vars
rm -rf .cargo
export LIBTORCH=/opt/conda/envs/dev/lib/python3.8/site-packages/torch
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
export LIBTORCH_CXX11_ABI=0
