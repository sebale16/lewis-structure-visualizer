#pragma once

// #include "display.hpp"

#include <memory>
#include <vector>
#include <string>
#include <optional>

#include <quaternion.h>

namespace molecule {

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

enum class Geometry {
    // 0 groups
    Single,
    // 1 group
    Linear2,
    // 2 groups
    Linear,
    // 3 groups
    TrigonalPlanar,
    Bent1Lone,
    // 4 groups
    Tetrahedral,
    TrigonalPyramidal,
    Bent2Lone,
    // 5 groups
    TrigonalBipyramidal,
    Seesaw,
    TShape,
    Linear3Lone,
    // 6 groups
    Octahedral,
    SquarePyramidal,
    SquarePlanar,
    // 7 groups
    PentagonalBipyramidal,
    PentagonalPyramidal,
    PentagonalPlanar,
    // 8 groups
    SquareAntiprismatic
};

enum class BondType {
    SIGMA,
    PI,
};

struct Atom {
    std::string name;
    // proton count in atom (used to make sphere representing atoms bigger/smaller)
    int protonCount;
    int id;
    int lone;
    // hybridization enum
    Hybridization hybridization;
    // number of p orbitals
    size_t pOrbitalCount;
};

class Molecule {
private:
    std::vector<std::shared_ptr<Atom>> atoms;
    // index matches with atoms
    std::vector<std::vector<std::pair<std::weak_ptr<Atom>, BondType>>> bondsWith;
    // pointer to central atom
    std::weak_ptr<Atom> centralAtom;
    // used to determine where to draw bonded atoms
    std::optional<Geometry> geometry;

    // computes geometry (Geometry) of model based on atoms and bonds_with and update `geometry`
    void ComputeGeometry();

public:
    // fills molecule with precomputed data from solver
    void FillMoleculeFromJSON(const std::string& filePath, const std::string& dataCSVPath);

    // computes location of center of atoms and their rotation, with central atom at origin; based on `group`
    // std::vector<std::tuple<std::weak_ptr<Atom>, display::Point, quaternion::Quaternion<float>>> ComputeAtomLocsRots();

    // geometry getter
    std::optional<Geometry> GetGeometry();
};

} // namespace model

// steps for displaying:
// 1. figure out center of central atom:
//      a. if molecule is monatomic, then center is (0, 0, 0)
//      b. if molecule is diatomic, then center is halfway between both atoms
// 2. each enum corresponding to hybridization will produce different layout of
// orbitals
// 3. to place non-central atoms, angle and distance from center will determine
// location of center of other atoms
//      a. the angle used to calculate rotation of orbitals will be used to
//          figure out the line on which to translate the atom from the origin
//          of the central atom
//      b. determine formula for rotation

// when zoomed out, element names should be written on the atoms
// start fading out when zooming in and then nucleus should be visible
