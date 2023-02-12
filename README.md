# egghead
*the world's smartest robot** 

## running
On x86/Nix systems, you only need to run `nix develop .` in the root to generate a suitable compilation environment. 

AArch64 build will use the same instructions, but require a native ARM-enabled PyTorch runtime.

### env

All systems require the following environment variables to run.

```bash
export LIBTORCH=$(python3 -c 'import torch; from pathlib import Path; print(Path(torch.__file__).parent)')
export DYLD_LIBRARY_PATH=${LIBTORCH}/lib
export LIBTORCH_CXX11_ABI=0
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
export DISCORD_TOKEN=YOUR-TOKEN-HERE
```
 
*Not actually worldly, smart or a robot (technically).
