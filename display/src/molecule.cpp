#include "molecule.hpp"
using namespace molecule;

#include "json.hpp"
#include <glm/gtc/constants.hpp>

#include <ranges>
#include <algorithm>
#include <fstream>
#include <sstream>
#include <unordered_map>
#include <format>
#include <print>

AtomMatrix BondedAtom::ToMatrix() const {
    std::println("Name: {}, Hybridization: {}", this->wPtrAtom.lock()->name, (int) this->wPtrAtom.lock()->hybridization);
    std::vector<std::pair<OrbitalType, glm::quat>> orbitalRots;
    // orientation is dependent on hybridization of the atom
    switch (this->wPtrAtom.lock()->hybridization) {
        case molecule::Hybridization::s:
            orbitalRots.emplace_back(OrbitalType::s, glm::quat());
            break;
        case molecule::Hybridization::sp:
            orbitalRots.emplace_back(OrbitalType::sp, glm::quat());
            orbitalRots.emplace_back(OrbitalType::sp, glm::angleAxis(glm::pi<float>(), glm::vec3(0, 0, 1)));
            break;
        case molecule::Hybridization::sp2:
            orbitalRots.emplace_back(OrbitalType::sp, glm::quat());
            orbitalRots.emplace_back(OrbitalType::sp, glm::angleAxis(2.f * glm::pi<float>() / 3.f, glm::vec3(0, 0, 1)));
            orbitalRots.emplace_back(OrbitalType::sp, glm::angleAxis(4.f * glm::pi<float>() / 3.f, glm::vec3(0, 0, 1)));
            break;
        case molecule::Hybridization::sp3:
            orbitalRots.emplace_back(OrbitalType::sp, glm::quat());
            orbitalRots.emplace_back(OrbitalType::sp, glm::angleAxis(static_cast<float>(glm::acos(-1./3.)), glm::vec3(0, 0, -1)));
            orbitalRots.emplace_back(OrbitalType::sp, glm::angleAxis(static_cast<float>(glm::acos(-1./3.)), glm::vec3(0, -sqrt(3)/2., 0.5)));
            orbitalRots.emplace_back(OrbitalType::sp, glm::angleAxis(static_cast<float>(glm::acos(-1./3.)), glm::vec3(0, sqrt(3)/2., 0.5)));
            break;
    }

    // handle p orbitals: perpendicular to sp orbital
    for (int i = 0; i < this->wPtrAtom.lock()->pOrbitalCount; i++) {
        orbitalRots.emplace_back(OrbitalType::p, glm::angleAxis(glm::pi<float>() / 2.f, glm::vec3(0, 1-i, i)));
    }

    return { .orbitals = orbitalRots };
}

Molecule::Molecule(const std::string& jsonPath, const std::string& dataCSVPath) {
    FillMoleculeFromJSON(jsonPath, dataCSVPath);
}

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
        if (aEntry["hybridization"] == "S") atom.hybridization = Hybridization::s;
        else if (aEntry["hybridization"] == "SP") atom.hybridization = Hybridization::sp;
        else if (aEntry["hybridization"] == "SP2") atom.hybridization = Hybridization::sp2;
        else if (aEntry["hybridization"] == "SP3") atom.hybridization = Hybridization::sp3;
        else if (aEntry["hybridization"] == "SP3D") atom.hybridization = Hybridization::sp3d;
        else if (aEntry["hybridization"] == "SP3D2") atom.hybridization = Hybridization::sp3d2;
        else if (aEntry["hybridization"] == "SP3D3") atom.hybridization = Hybridization::sp3d3;
        else if (aEntry["hybridization"] == "SP3D4") atom.hybridization = Hybridization::sp3d4;
        else if (aEntry["hybridization"] == "SP3D5") atom.hybridization = Hybridization::sp3d5;
        else throw std::runtime_error(
                std::format("Atom with name {} and id {} had invalid hybridization entry of {}.",
                    atom.name, atom.id, std::string(aEntry["hybridization"])));

        atoms.push_back(std::make_shared<Atom>(atom));
    }

    // now that atoms are there, construct bonds between atoms
    // loop through each atom entry
    for (size_t i = 0; i < atomsEntries.size(); i++) {
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
                bonds.emplace_back(std::weak_ptr<Atom>(bondedWithAtom), bondType);
            } else {
                throw std::runtime_error(
                        std::format("Atom with name {} and id {} could not find atom to bond with.",
                            atoms[i]->name, atoms[i]->id));
            }
        }
        bondsWith.push_back(bonds);
    }
}

std::expected<Geometry, std::string> Molecule::ComputeGeometry() {
    // find number of atoms
    auto atomCount = atoms.size();

    // base cases
    if (atomCount == 1) {
        centralAtom = atoms[0];
        return Geometry::Single;
    } else if (atomCount == 2) {
        // finds atom in the list with greatest protonCount
        auto maxProtonCountAtom = std::max_element(atoms.begin(), atoms.end(),
                [](const auto& a1, const auto& a2) {
                    return a1->protonCount < a2->protonCount;
                }
        );
        if (maxProtonCountAtom != atoms.end()) {
            centralAtom = *maxProtonCountAtom;
            return Geometry::Linear2;
        } else {
            return std::unexpected("Could not determine centralAtom for this molecule!");
        }
    }

    size_t maxCount = 0;
    for (size_t i = 0; i < atomCount; i++) {
        auto currAtom = atoms[i]; // shared_ptr
        size_t count = 0;
        for (size_t j = 0; j < atomCount; j++) {
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

    // return geometry depending on those two values
    int stericNumber = atomCount - 1 + loneCount / 2;
    switch (stericNumber) {
        case 2:
            return Geometry::Linear;
        case 3:
            if (loneCount == 0) return Geometry::TrigonalPlanar;
            else if (loneCount == 2) return Geometry::Bent1Lone;
        case 4:
            if (loneCount == 0) return Geometry::Tetrahedral;
            else if (loneCount == 2) return Geometry::TrigonalPyramidal;
            else if (loneCount == 4) return Geometry::Bent2Lone;
        case 5:
            if (loneCount == 0) return Geometry::TrigonalBipyramidal;
            else if (loneCount == 2) return Geometry::Seesaw;
            else if (loneCount == 4) return Geometry::TShape;
            else if (loneCount == 6) return Geometry::Linear3Lone;
        case 6:
            if (loneCount == 0) return Geometry::Octahedral;
            else if (loneCount == 2) return Geometry::SquarePyramidal;
            else if (loneCount == 4) return Geometry::SquarePlanar;
        case 7:
            if (loneCount == 0) return Geometry::PentagonalBipyramidal;
            else if (loneCount == 2) return Geometry::PentagonalPyramidal;
            else if (loneCount == 4) return Geometry::PentagonalPlanar;
        case 8:
            if (loneCount == 0) return Geometry::SquareAntiprismatic;
        default:
            return std::unexpected("No geometry found for molecule!");
    }
}

std::expected<std::vector<BondedAtom>, std::string> Molecule::ComputeAtomLocsRots() {
    if (this->atoms.size() == 0) return std::unexpected("Cannot call on empty molecule!");
    auto geometry = this->ComputeGeometry().value();
    std::println("Geometry: {}", (int) geometry);

    // place centralAtom at center
    auto bondedCentralAtom = BondedAtom {
        .wPtrAtom = centralAtom,
        .loc = glm::vec3(0.f),
        .rot = glm::identity<glm::quat>(),
    };

    // view over elements that are not the central atom
    auto nonCentralAtomsFilter = atoms | std::views::filter([this](const auto& a) {
        return centralAtom.owner_before(a) || a.owner_before(centralAtom);
    });

    if (geometry == Geometry::Single) {
        return std::expected<std::vector<BondedAtom>, std::string>({ bondedCentralAtom });
    } else if (geometry == Geometry::Linear2) {
        // second atom location will depend on hybridization of centralAtom and its own hybridization
        // since one orbital will always be in positive x direction, if second atom is not hybridized,
        // then center of second atom should be slightly more in positive x direction, depending on protonCount of second atom
        // if second atom is hybridized, second atom should be at such a distance so that orbitals are overlapping and symmetric

        if (!nonCentralAtomsFilter.empty()) {
            auto secondAtom = nonCentralAtomsFilter.front();
            auto bondedSecondAtom = BondedAtom { .wPtrAtom = std::weak_ptr<Atom>(secondAtom) };
            if (secondAtom->hybridization == Hybridization::s) {
                // x is shifted by a multiplier
                bondedSecondAtom.loc = glm::vec3(-(S_ORBITAL_SHIFT * (secondAtom->protonCount)), 0.f, 0.f);
                bondedSecondAtom.rot = glm::identity<glm::quat>();
            } else {
                // x is shifted; also second atom must be rotated so that one of its orbitals points in the -x direction
                bondedSecondAtom.loc = glm::vec3(-SP_ORBITAL_SHIFT, 0.f, 0.f);
                bondedSecondAtom.rot = glm::angleAxis(glm::pi<float>(), glm::vec3(0.f, 0.f, 1.f));
            }
            // move both atoms so that halfway distance is at (0,0,0) if CENTRALIZE
#if CENTRALIZE
            auto distance = bondedSecondAtom.loc.x;
            bondedCentralAtom.loc.x -= distance / 2.f;
            bondedSecondAtom.loc.x -= distance / 2.f;
#endif
            return std::expected<std::vector<BondedAtom>, std::string>({
                    bondedCentralAtom, bondedSecondAtom
            });
        } else return std::unexpected("Could not find second atom!");
    } else {
        if (!nonCentralAtomsFilter.empty()) {
            std::vector<BondedAtom> bondedAtoms = {bondedCentralAtom};
            for (auto [index, atom] : nonCentralAtomsFilter | std::views::enumerate) {
                auto shift = atom->hybridization == Hybridization::s ? S_ORBITAL_SHIFT : SP_ORBITAL_SHIFT;
                switch (geometry) {
                    case Geometry::Linear:
                        bondedAtoms.emplace_back(
                            atom,
                            glm::vec3(-pow(-1, index) * shift, 0.f, 0.f),
                            glm::angleAxis((index + 1) * glm::pi<float>(), glm::vec3(0.f, 0.f, 1.f))
                                 * glm::angleAxis((index - 1) * glm::pi<float>() / 2.f, glm::vec3(1.f, 0.f, 0.f)) // additional rotation so that p orbitals line up
                        );
                        break;
                    case Geometry::TrigonalPlanar:
                    case Geometry::Bent1Lone: {
                        glm::quat rot = glm::angleAxis(2.f * index * glm::pi<float>() / 3.f, glm::vec3(0.f, 0.f, 1.f));
                        bondedAtoms.emplace_back(
                            atom,
                            rot * glm::vec3(-shift, 0.f, 0.f),
                            rot * glm::angleAxis(glm::pi<float>(), glm::vec3(0, 0, 1))
                        );
                        break;
                    }
                    case Geometry::Tetrahedral:
                    case Geometry::TrigonalPyramidal:
                    case Geometry::Bent2Lone: {
                        glm::quat rot = glm::identity<glm::quat>();
                        if (index == 1) {
                            rot = glm::angleAxis(static_cast<float>(glm::acos(-1./3.)), glm::vec3(0.f, 0.f, -1.f));
                        } else if (index == 2) {
                            rot = glm::angleAxis(static_cast<float>(glm::acos(-1./3.)), glm::vec3(0, -sqrt(3)/2., 0.5));
                        } else if (index == 3) {
                            rot = glm::angleAxis(static_cast<float>(glm::acos(-1./3.)), glm::vec3(0, sqrt(3)/2., 0.5));
                        }
                        bondedAtoms.emplace_back(
                            atom,
                            rot * glm::vec3(-shift, 0.f, 0.f),
                            rot * glm::angleAxis(glm::pi<float>(), glm::vec3(0, 0, 1))
                        );
                        break;
                    }
                }
            }
            return bondedAtoms;
        } else return std::unexpected("Could not find non-central atoms!");
    }
}
