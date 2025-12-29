#pragma once

#include <memory>
#include <vector>

#include <quaternion.h>
#include <nlohmann/json.hpp>

#include "display.hpp"

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

enum class Group {
    Single, // one atom
    Linear2,
    Linear3,
    TriPlanar,
    Tetra,
    TriBipyr,
    Octa,
    PentBypir
};

enum class Bond {
    SIGMA,
    PI,
};

struct Atom {
    // proton count in atom (used to make sphere representing atoms bigger/smaller)
    uint8_t protonCount;
    uint8_t id;
    uint8_t lone;
    // hybridization enum
    Hybridization hybridization;
    // number of p orbitals
    uint8_t pOrbitalCount;
};

class Molecule {
private:
    std::vector<std::shared_ptr<Atom>> atoms;
    // index matches with atoms
    std::vector<std::vector<std::pair<std::weak_ptr<Atom>, Bond>>> bondsWith;
    // used to determine where to draw bonded atoms
    Group group;

    // computes geometry (Group) of model based on atoms and bonds_with and updates `group`
    void computeGroup();

public:
    Molecule(std::vector<std::shared_ptr<Atom>>& atoms_) : atoms(atoms_) {}

    // fills molecule with precomputed data from solver
    void fillMolecule(nlohmann::json data);

    // computes location of center of atoms and their rotation, with central atom at origin; based on `group`
    std::vector<std::tuple<std::weak_ptr<Atom>, display::Point, quaternion::Quaternion<float>>> computeAtomLocsRots();
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
