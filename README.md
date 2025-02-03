Built in Rust, this is a Lewis structure visualizer for simple compounds in which there is a central atom to which every other atom is bonded to. Calculations use basic Lewis structure building rules, i.e., each atom wants a full octet and formal charge is minimized.
To do: implement hybridization calculation (possibly even switching to using hybridization to determine bonds); show number of sigma and pi bonds; eventually 3d.

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
