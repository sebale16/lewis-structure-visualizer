Given:
```
enum class Hybridization {
    s,
    sp,
    sp2,
    sp3,
    sp3d,
    sp3d2,
    sp3d3,
    sp3d4,
    sp3d5,
};
``` 
Define standard oriention of each orbital per hybridization such that one sp orbital points to the right (x-axis)
and other orbitals are placed following that. For n orbitals, each other orbital then has degree of rotation 
and axis around which to rotate, which is stored in vector of size n - 1 where each element is quaternion.
In the following table, axis vector is defined with unit vectors i, j, k pointing in directions of x, y, z axes respectively.
z-axis points out of the screen, x points to the right of the screen, y points up. 
| Hybridization | s orbital count | sp orbital count | quat vec (axis vec, angle) |
| ------------- | --------------- | ---------------- | -------- |
| S | 1 | 0 | - |
| SP | 0 | 2 | {(k, pi)} |
| SP2 | 0 | 3 | {(k, 2pi/3), (k, 4pi/3)} |
| SP3 | 0 | 4 | {(-k, arccos(-1/3)), (-sqrt(3)/2\*j+0.5\*k, arccos(-1/3)), (sqrt(3)/2\*j+0.5\*k, arccos(-1/3))} |
