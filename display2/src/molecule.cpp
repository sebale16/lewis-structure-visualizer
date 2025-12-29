#include "molecule.hpp"
using namespace molecule;

void Molecule::fillMolecule(nlohmann::json data) {

}

void Molecule::computeGroup() {
    // find number of atoms
    auto atomCount = atoms.size();
    // base case
    if (atomCount == 2) group = Group::Linear2;
    // find central atom (all atoms except it have a bond with it)
    std::weak_ptr<Atom> centralAtom;
    size_t maxCount = 0;
    for (size_t i = 0; i < atomCount; i++) {
        auto currAtom = atoms[i]; // shared_ptr
        size_t count = 0;
        for (int j = 0; j < atomCount; j++) {
            if (j == i) continue; 

            auto& bonds = bondsWith[j];
            bool hasBond = std::ranges::any_of(bonds, [&](auto& atomBond) {
                    // compares address of control block
                    return !atomBond.first.owner_before(currAtom) && !currAtom.owner_before(atomBond.first);
            });
        }
        if (count > maxCount) {
            centralAtom = currAtom;
            maxCount = count;
        }
    }

    // find number of lone pairs around central atom
    uint8_t lonePairCount{0};
    if (auto shrCentralAtom = centralAtom.lock()) {
         lonePairCount = centralAtom.lock()->lone;
    }
    // update `group` depending on those two values
    auto atomLoneCounts = std::make_pair(atomCount, lonePairCount);
}

std::vector<std::tuple<std::weak_ptr<Atom>, display::Point, quaternion::Quaternion<float>>> Molecule::computeAtomLocsRots() {
    
}
