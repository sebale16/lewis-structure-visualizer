#pragma once

#include "config.hpp"

#include <memory>
#include <vector>
#include <string>
#include <expected>

// used to define how much should S hybridized atom be shifted depending on its protonCount
#ifndef S_ORBITAL_SHIFT
#define S_ORBITAL_SHIFT 2.f
#endif

// how far away atoms are from central atom if bond is hybridized
#ifndef SP_ORBITAL_SHIFT
#define SP_ORBITAL_SHIFT 3.f
#endif

// scale S orbital model
#ifndef S_ORBITAL_SCALE
#define S_ORBITAL_SCALE 0.25f
#endif

// scale SP orbital model
#ifndef SP_ORBITAL_SCALE
#define SP_ORBITAL_SCALE 1.f
#endif

// scale P orbital model
#ifndef P_ORBITAL_SCALE
#define P_ORBITAL_SCALE 1.f
#endif

// should molecule with 2 atoms be placed so that midway point is at (0,0,0)
#ifndef CENTRALIZE
#define CENTRALIZE 1
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

enum class OrbitalType {
    s,
    sp,
    p,
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

    // function to transform an atom into its matrix represention: 
    // each element of the vector is a pair of an orbital type with its orientation
    std::vector<std::pair<OrbitalType, glm::quat>> ToMatrix() const;
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
    // constructor for filling through json
    Molecule(const std::string& jsonPath, const std::string& dataCSVPath);

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

// when zoomed out, element names should be written on the atoms
// start fading out when zooming in and then nucleus should be visible
