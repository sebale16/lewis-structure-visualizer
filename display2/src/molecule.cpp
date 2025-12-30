#include "molecule.hpp"
using namespace molecule;

#include <algorithm>
#include <fstream>
#include <sstream>
#include <unordered_map>
#include <format>
#include <iostream>

#include <nlohmann/json.hpp>

void Molecule::FillMoleculeFromJSON(const std::string& jsonPath, const std::string& dataCSVPath) {
    std::ifstream csvFile(dataCSVPath);
    if (!csvFile.is_open()) {
        throw std::runtime_error("Could not open file: " + dataCSVPath);
    }

    // create map of element to atomic number (used to set protonCount in Atom)
    std::unordered_map<std::string, int> elementToProton;

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
                int protonNum = std::stoi(protonStr);

                // strip spaces from elementStr
                std::erase(elementStr, ' ');
                elementToProton[elementStr] = protonNum;
            } catch (const std::exception& e) {
                throw std::runtime_error("Error building map: " + std::string(e.what()));
            }
        }
    }

    std::ifstream file(jsonPath);
    if (!file.is_open()) {
        throw std::runtime_error("Could not open file: " + jsonPath);
    }

    nlohmann::json data;
    file >> data;

    // fill `atoms` and `bondsWith` with data
    auto& atomsEntries = data["atoms"];
    for (const auto& aEntry : atomsEntries) {
        // construct Atom
        Atom atom = Atom {
            .id = aEntry["id"],
            .lone = aEntry["lone"],
            .pOrbitalCount = aEntry["p_orbitals"].size(),
        };
        // remove spaces from string
        std::string aName = aEntry["name"];
        std::erase(aName, ' ');
        atom.name = aName;
        atom.protonCount = elementToProton[aName];

        // match hybridization from data to enum
        if (aEntry["hybridization"] == "S") atom.hybridization = Hybridization::S;
        else if (aEntry["hybridization"] == "SP") atom.hybridization = Hybridization::SP;
        else if (aEntry["hybridization"] == "SP2") atom.hybridization = Hybridization::SP2;
        else if (aEntry["hybridization"] == "SP3") atom.hybridization = Hybridization::SP3;
        else if (aEntry["hybridization"] == "SP3D") atom.hybridization = Hybridization::SP3D;
        else if (aEntry["hybridization"] == "SP3D2") atom.hybridization = Hybridization::SP3D2;
        else if (aEntry["hybridization"] == "SP3D3") atom.hybridization = Hybridization::SP3D3;
        else if (aEntry["hybridization"] == "SP3D4") atom.hybridization = Hybridization::SP3D4;
        else if (aEntry["hybridization"] == "SP3D5") atom.hybridization = Hybridization::SP3D5;
        else throw std::runtime_error(
                std::format("Atom with name {} and id {} had invalid hybridization entry of {}.",
                    atom.name, atom.id, std::string(aEntry["hybridization"])));

        atoms.emplace_back(std::make_shared<Atom>(atom));
    }

    // now that atoms are there, construct bonds between atoms
    // loop through each atom entry
    for (int i = 0; i < atomsEntries.size(); i++) {
        std::vector<std::pair<std::weak_ptr<Atom>, BondType>> bonds;
        auto& bondsWithThisAtom = atomsEntries[i]["bonds_with"];
        // loop through all bonds
        for (const auto& bEntry : bondsWithThisAtom) {
            std::string bName = bEntry["name"];
            std::erase(bName, ' ');
            // find if entry in bond matches with an atom
            auto itBondedWithAtom = std::find_if(atoms.begin(), atoms.end(), [&](const std::shared_ptr<Atom>& a) {
                    return a->id == bEntry["id"] && a->name == bName;
            });
            // continue only if found a valid atom
            if (itBondedWithAtom != atoms.end()) {
                auto bondedWithAtom = std::weak_ptr<Atom>(*itBondedWithAtom);
                // match bond string with bond enum
                BondType bondType;
                if (bEntry["bond_type"] == "SIGMA") bondType = BondType::SIGMA;
                else if (bEntry["bond_type"] == "PI") bondType = BondType::PI;
                else throw std::runtime_error(
                        std::format("Atom with name {} and id {} had invalid bond_type entry of {}.",
                            atoms[i]->name, atoms[i]->id, std::string(bEntry["bond_type"])));
                // create bond with that atom
                bonds.emplace_back(std::make_pair(std::weak_ptr<Atom>(bondedWithAtom), bondType));
            } else {
                throw std::runtime_error(
                        std::format("Atom with name {} and id {} could not find atom to bond with.",
                            atoms[i]->name, atoms[i]->id));
            }
        }
        bondsWith.push_back(bonds);
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
            if (hasBond) count++;
        }
        if (count > maxCount) {
            centralAtom = currAtom;
            maxCount = count;
        }
    }

    // find number of lone pairs around central atom
    int loneCount{0};
    if (auto shrCentralAtom = centralAtom.lock()) {
         loneCount = shrCentralAtom->lone;
    }

    // update `group` depending on those two values
    // base cases
    if (atomCount == 1) geometry = Geometry::Single;
    else if (atomCount == 2) geometry = Geometry::Linear2;
    else {
        int stericNumber = atomCount - 1 + loneCount / 2;
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
                else if (loneCount == 6) geometry = Geometry::Linear3Lone;
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

//std::vector<std::tuple<std::weak_ptr<Atom>, display::Point, quaternion::Quaternion<float>>> Molecule::ComputeAtomLocsRots() {
//    
//}

std::optional<Geometry> Molecule::GetGeometry() {
    if (!geometry.has_value()) ComputeGeometry();
    return geometry;
}
