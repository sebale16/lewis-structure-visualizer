#pragma once

#include "config.hpp"

#include <memory>
#include <vector>
#include <string>
#include <expected>

// used to define how much should S hybridized atom be shifted depending on its protonCount
#ifndef S_ORBITAL_SHIFT
#define S_ORBITAL_SHIFT 1.f
#endif

// how far away atoms are from central atom if bond is hybridized
#ifndef SP_ORBITAL_SHIFT
#define SP_ORBITAL_SHIFT 2.f
#endif

// scale S orbital model
#ifndef S_ORBITAL_SCALE
#define S_ORBITAL_SCALE 1.f
#endif

// scale SP orbital model
#ifndef SP_ORBITAL_SCALE
#define SP_ORBITAL_SCALE 1.f
#endif

// should molecule with 2 atoms be placed so that midway point is at (0,0,0)
#ifndef CENTRALIZE
#define CENTRALIZE 0
#endif

namespace molecule {

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

// atom itself does not have rotation trait
// a given hybridization produces same orientation of orbitals as outlined in docs
// rotation is given by ComputeAtomLocsRots and saved in BondedAtom
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

// represents bonded atom in a molecule
// an Atom with position and rotation given by ComputeAtomLocsRots
struct BondedAtom {
    std::weak_ptr<Atom> wPtrAtom;
    glm::vec3 loc;
    glm::quat rot;
};

class Molecule {
private:
    std::vector<std::shared_ptr<Atom>> atoms;
    // index matches with atoms
    std::vector<std::vector<std::pair<std::weak_ptr<Atom>, BondType>>> bondsWith;
    // pointer to central atom, determined in ComputeGeometry;
    // if atomCount == 2, centralAtom will be atom with greater protonCount
    std::weak_ptr<Atom> centralAtom;

public:
    // fills molecule with precomputed data from solver
    void FillMoleculeFromJSON(const std::string& jsonPath, const std::string& dataCSVPath);

    // computes geometry (Geometry) of model based on atoms and bonds_with;
    // returns error message if no valid geometry or if called on empty molecule
    std::expected<Geometry, std::string> ComputeGeometry();

    // computes location of center of atoms and their rotation, with central atom at origin; based on geometry;
    // returns error message if called on empty molecule
    std::expected<std::vector<BondedAtom>, std::string> ComputeAtomLocsRots();
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
