#include "molecule.hpp"

#include <iostream>

int main() {
    std::string jsonFilePath = "/home/seb/projects/lewis-structure-visualizer/solver/out/N2_0.json";
    std::string csvFilePath = "/home/seb/projects/lewis-structure-visualizer/data/data.csv";

    molecule::Molecule mol;
    mol.FillMoleculeFromJSON(jsonFilePath, csvFilePath);
    auto geo = static_cast<int>(mol.ComputeGeometry().value());
    std::cout << geo << "\n";

    auto locsRots = mol.ComputeAtomLocsRots().value();
    for (auto a : locsRots) {
        std::cout << a.loc.x << " " << a.loc.y << " " << a.loc.z << "\n";
        std::cout << a.rot.x << " " << a.rot.y << " " << a.rot.z << "\n";
    }
    return 0;
}
