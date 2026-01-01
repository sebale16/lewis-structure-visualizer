Built using Rust, this is a Lewis structure visualizer for simple compounds in which there is a central atom to which every other atom is bonded to. Calculations use basic Lewis structure building rules, i.e., each atom wants a full octet and formal charge is minimized.

Done: implement hybridization calculation (possibly even switching to using hybridization to determine bonds); show number of sigma and pi bonds.
 
To do: 3D.

The code that is in the `master` branch and has this display feature is my outdated code, which has a worse method for determining Lewis structures than the new code in the `display` and `display2` branches that do not have a nice display output but which are for the 3D graphics pipeline. `display` is my unfinished work with the `wgpu` crate while `display2` is my ongoing work using Google's Dawn WebGPU implementation.

# Usage
```
cargo run COMPOUND CHARGE
```
## Examples:
### Carbon dioxide
Input:
```
cargo run CO2 0
```
Output:
```
Linear
 ..      ..
 O ==C ==O
 ˙˙      ˙˙
```

### Sulfate
Input:
```
cargo run SO4 -2
```
Output:
```
Tetrahedral
    :O :
 ..  ||  ..
:O --S ==O
 ˙˙  |   ˙˙
    :O :
     ˙˙
```

### Sulfur tetrafluoride
Input:
```
cargo run SF4 0
```
Output:
```
Seesaw
 ..  ..  ..
:F --S --F :
 ˙˙ / \  ˙˙
   /   \
 :F :  :F :
  ˙˙    ˙˙
```
