#include "molecule.hpp"
using namespace molecule;

#include <algorithm>
#include <fstream>
#include <sstream>
#include <unordered_map>

#include <nlohmann/json.hpp>

void Molecule::FillMoleculeFromJSON(const std::string& filePath, const std::string& dataCSVPath) {
    std::ifstream csvFile(dataCSVPath);
    if (!csvFile.is_open()) {
        throw std::runtime_error("Could not open file: " + dataCSVPath);
    }

    // create map of element to atomic number (used to set protonCount in Atom)
    std::unordered_map<std::string, uint8_t> elementToProton;

    std::string line;
    // skip header
    std::getline(csvFile, line);

    // fill map with data from csv, line by line
    while (std::getline(csvFile, line)) {
        std::stringstream ss(line);
        std::string elementStr, protonStr;

        // get first and second columns (atomic number is first column, symbol is second column)
        if (std::getline(ss, protonStr, ',') && std::getline(ss, elementStr, ',')) {
            try {
                // convert protonStr to int to be placed in map
                uint8_t protonNum = std::stoi(protonStr);
                elementToProton[elementStr] = protonNum;
            } catch (const std::exception& e) {
                throw std::runtime_error("Error building map: " + std::string(e.what()));
            }
        }
    }

    std::ifstream file(filePath);
    if (!file.is_open()) {
        throw std::runtime_error("Could not open file: " + filePath);
    }

    nlohmann::json data;
    file >> data;
    
    auto& atomsEntries = data["atoms"];
    for (const auto& aEntry : atomsEntries) {
        Atom atom;
        auto shrAtom = std::make_shared<Atom>(atom);
    }
}

void Molecule::ComputeGeometry() {
    // find number of atoms
    auto atomCount = atoms.size();
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
    uint8_t loneCount{0};
    if (auto shrCentralAtom = centralAtom.lock()) {
         loneCount = centralAtom.lock()->lone;
    }

    // update `group` depending on those two values
    // base cases
    if (atomCount == 1) geometry = Geometry::Single;
    else if (atomCount == 2) geometry = Geometry::Linear2;
    else {
        uint8_t stericNumber = atomCount - 1 + loneCount / 2;
        switch (stericNumber) {
            case 2:
                geometry = Geometry::Linear;
                break;
            case 3:
                if (loneCount == 0) geometry = Geometry::TrigonalPlanar;
                else if (loneCount == 2) geometry = Geometry::Bent1Lone;
                break;
            case 4:
                if (loneCount == 0) geometry = Geometry::Tetrahedral;
                else if (loneCount == 2) geometry = Geometry::TrigonalPyramidal;
                else if (loneCount == 4) geometry = Geometry::Bent2Lone;
                break;
            case 5:
                if (loneCount == 0) geometry = Geometry::TrigonalBipyramidal;
                else if (loneCount == 2) geometry = Geometry::Seesaw;
                else if (loneCount == 4) geometry = Geometry::TShape;
                else if (loneCount == 6) geometry = Geometry::Linear;
                break;
            case 6:
                if (loneCount == 0) geometry = Geometry::Octahedral;
                else if (loneCount == 2) geometry = Geometry::SquarePyramidal;
                else if (loneCount == 4) geometry = Geometry::SquarePlanar;
                break;
            case 7:
                if (loneCount == 0) geometry = Geometry::PentagonalBipyramidal;
                else if (loneCount == 2) geometry = Geometry::PentagonalPyramidal;
                else if (loneCount == 4) geometry = Geometry::PentagonalPlanar;
                break;
            case 8:
                if (loneCount == 0) geometry = Geometry::SquareAntiprismatic;
                break;
        }
        // if no case is fired, then geometry will stay nullopt
    }
}

std::vector<std::tuple<std::weak_ptr<Atom>, display::Point, quaternion::Quaternion<float>>> Molecule::ComputeAtomLocsRots() {
    
}
