#include "molecule.hpp"

#include <iostream>

int main() {
    std::string jsonFilePath = "/home/seb/projects/lewis-structure-visualizer/solver/out/N2_0.json";
    std::string csvFilePath = "/home/seb/projects/lewis-structure-visualizer/data/data.csv";

    molecule::Molecule mol;
    mol.FillMoleculeFromJSON(jsonFilePath, csvFilePath);
    auto geo = static_cast<int>(mol.ComputeGeometry().value());
    std::cout << geo << "\n";
    return 0;
}
