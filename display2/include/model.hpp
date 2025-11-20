#pragma once

#include <vector>
#include <memory>

#include <third_party/quaternions/include/quaternion.h>

namespace model {

enum class Hybridization {
    S,
    SP,
    SP2,
    SP3,
    SP2D,
    SP3D,
    SP3D2,
    SP3D3,
    SP3D4,
    SP3D5,
};

struct Atom {
    // proton count in atom (used to make atoms bigger/smaller)
    uint8_t protonCount;
    // hybridization enum
    Hybridization hybridization;
    // quaternion corresponding to rotation
    quaternion::Quaternion<float> rotation;
};

class Model {
private:
    std::vector<std::shared_ptr<Atom>> molecule;
};

}

// steps for displaying:
// 1. figure out center of central atom:
//      a. if molecule is monatomic, then center is (0, 0, 0)
//      b. if molecule is diatomic, then center is halfway between both atoms
// 2. each enum corresponding to hybridization will produce different layout of orbitals
// 3. to place non-central atoms, angle and distance from center will determine location of center of other atoms
//      a. the angle used to calculate rotation of orbitals will be used to 
//          figure out the line on which to translate the atom from the origin of the central atom
//      b. determine formula for rotation

// when zoomed out, element names should be written on the atoms
// start fading out when zooming in and then nucleus should be visible
