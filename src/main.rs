use std::cell::RefCell;
use std::env;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::zip;
use std::rc::Rc;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Element {
    name: String,
    valence: i64,
    electroneg: i64,
    id: i64
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Atom {
    name: String,
    valence: i64,
    lone: i64,
    id: i64
}

impl Atom {
    fn new(name: &str, valence: i64, lone: i64, id: i64) -> AtomRef {
        Rc::new(RefCell::new(Atom {
            name: name.to_string(),
            valence,
            lone,
            id,
        }))
    }

    fn get_bonded_electrons_count(&self, bonds_with: &HashMap<Atom, BondType>) -> i64 {
        bonds_with.iter().map(|(_, bond_type)| {
            match bond_type {
                BondType::SINGLE => 1,
                BondType::DOUBLE => 2,
                BondType::TRIPLE => 3
            }
        }).sum()
    }

    fn formal_charge(&self, bonds_with: &HashMap<Atom, BondType>) -> i64 {
        self.valence - self.lone - self.get_bonded_electrons_count(&bonds_with)
    }

    fn electrons_to_octet(&self, bonds_with: &HashMap<Atom, BondType>) -> i64 {
        8 - self.lone - self.get_bonded_electrons_count(&bonds_with) * 2
    }

    fn electrons_to_two(&self, bonds_with: &HashMap<Atom, BondType>) -> i64 {
        2 - self.lone - self.get_bonded_electrons_count(&bonds_with) * 2
    }
}


#[derive(Clone, Copy, Debug)]
enum BondType {
    SINGLE,
    DOUBLE,
    TRIPLE
}

struct ParsedCompound {
    elements: HashSet<Element>, // set corresponding to each element
    charge: i64
}

#[derive(Debug, Clone)]
struct ModelCompound {
    adjacency_list: HashMap<Atom, HashMap<Atom, BondType>>
}

impl ModelCompound {
    fn new() -> Self {
        ModelCompound {
            adjacency_list: HashMap::new()
        }
    }

    fn add_atom(&mut self, atom: &Atom) {
        self.adjacency_list.entry(atom.clone()).or_insert(HashMap::new());
    }

    // fn remove_bond(&mut self, atom_1_name: &str, atom_1_id: i64, atom_2_name: &str, atom_2_id: i64) -> BondType {
    //     let mut curr_model = self.adjacency_list.clone();
    //
    //     let atom_1 = self.adjacency_list.clone().keys().filter(|a| {
    //         a.name == atom_1_name && a.id == atom_1_id
    //     }).next().unwrap().clone();
    //
    //     let atom_2 = self.adjacency_list.clone().keys().filter(|a| {
    //         a.name == atom_2_name && a.id == atom_2_id
    //     }).next().unwrap().clone();
    //
    //     let bond = self.adjacency_list.get(&atom_1).unwrap().get(&atom_2).unwrap();
    //
    //     // remove atom_2 from atom_1
    //     self.adjacency_list.get(&atom_1).unwrap().remove(&atom_2);
    //
    //     // get atom_1 bonds
    //     let atom_1_bonds_or_none = self.adjacency_list.get(&atom_1);
    //
    //     // remove atom_1 from atom_2
    //     self.adjacency_list.get(&atom_2).unwrap().remove(&atom_1);
    //
    //     // get atom_2 bonds
    //     let atom_2_bonds_or_none = self.adjacency_list.get(&atom_1);
    //
    //     let mut atom_1_new = atom_1.clone();
    //     let mut atom_2_new = atom_2.clone();
    //
    //     // add electrons back corresponding to bond
    //     match bond {
    //         BondType::SINGLE => {
    //             atom_1_new.lone += 1;
    //             atom_2_new.lone += 1;
    //         }
    //         BondType::DOUBLE => {
    //             atom_1_new.lone += 2;
    //             atom_2_new.lone += 2;
    //         }
    //         BondType::TRIPLE => {
    //             atom_1_new.lone += 3;
    //             atom_2_new.lone += 3;
    //         }
    //     }
    //
    //     // change atoms that are bonded with atom_1 to show atom_1_new
    //     match atom_1_bonds_or_none {
    //         Some(mut atom_1_bonds) => {
    //             // loop through each atom that is bonded with atom_1, remove old atom_1 and insert atom_1_new with old bond type
    //             for (atom, bond_type) in atom_1_bonds {
    //                 self.adjacency_list.get(&atom).unwrap().remove(&atom_1);
    //                 self.adjacency_list.get(&atom).unwrap().entry(atom_1_new.clone()).insert_entry(bond_type.clone());
    //             }
    //         }
    //         None => {}
    //     }
    //
    //     match atom_2_bonds_or_none {
    //         Some(mut atom_2_bonds) => {
    //             // loop through each atom that is bonded with atom_2, remove old atom_2 and insert atom_2_new with old bond type
    //             for (atom, bond_type) in atom_2_bonds {
    //                 self.adjacency_list.get(&atom).unwrap().remove(&atom_2);
    //                 self.adjacency_list.get(&atom).unwrap().entry(atom_2_new.clone()).insert_entry(bond_type.clone());
    //             }
    //         }
    //         None => {}
    //     }
    //
    //     *bond
    // }

    fn add_bond(&mut self, atom_1_name: &str, atom_1_id: i64, atom_2_name: &str, atom_2_id: i64, bond_type: BondType) -> bool {
        let mut atom_1 = self.adjacency_list.clone().keys().filter(|a| {
            a.name == atom_1_name && a.id == atom_1_id
        }).next().unwrap().clone();

        let mut atom_2 = self.adjacency_list.clone().keys().filter(|a| {
            a.name == atom_2_name && a.id == atom_2_id
        }).next().unwrap().clone();

        let bonds_1_or_none = self.adjacency_list.remove(&atom_1);
        let bonds_2_or_none = self.adjacency_list.remove(&atom_2);

        let atom_1_old = atom_1.clone();
        let atom_2_old = atom_2.clone();

        let mut bond_allowed = false;

        match bond_type {
            BondType::SINGLE => {
                if atom_1.lone > 0 && atom_2.lone > 0 {
                    bond_allowed = true;
                    atom_1.lone -= 1;
                    atom_2.lone -= 1;
                }
            }
            BondType::DOUBLE => {
                if atom_1.lone > 1 && atom_2.lone > 1 {
                    bond_allowed = true;
                    atom_1.lone -= 1;
                    atom_2.lone -= 1;
                }
            }
            BondType::TRIPLE => {
                if atom_1.lone > 2 && atom_2.lone > 2 {
                    bond_allowed = true;
                    atom_1.lone -= 1;
                    atom_2.lone -= 1;
                }
            }
        }

        if bond_allowed {
            match bonds_1_or_none {
                Some(mut bonds) => {

                    // get bonds with old atom_2
                    let mut old_bonds = self.adjacency_list.clone();
                    let filter = old_bonds.iter_mut().filter(|(_, b)| {
                        b.keys().any(|x| {
                            x.name == atom_2_old.name && x.id == atom_2_old.id
                        })
                    });

                    // add atom_2
                    bonds.remove(&atom_2_old.clone());
                    bonds.entry(atom_2.clone()).insert_entry(bond_type);
                    self.adjacency_list.entry(atom_1.clone()).insert_entry(bonds);

                    // revise old bonds to show updated atom_2
                    for (atom, bonds_1) in filter {
                        let bond_old_type = bonds_1.remove(&atom_2_old).unwrap();
                        bonds_1.entry(atom_2.clone()).insert_entry(bond_old_type);
                        self.adjacency_list.entry(atom.clone()).insert_entry(bonds_1.clone());
                    }

                }
                None => {
                    self.adjacency_list.entry(atom_1.clone()).or_insert_with(HashMap::new).insert(atom_2.clone(), bond_type);
                }
            }

            match bonds_2_or_none {
                Some(mut bonds) => {
                    // get bonds with old atom_1
                    let mut old_bonds = self.adjacency_list.clone();
                    let filter = old_bonds.iter_mut().filter(|(_, b)| {
                        b.keys().any(|x| {
                            x.name == atom_1_old.name && x.id == atom_1_old.id
                        })
                    });

                    // add atom_1
                    bonds.remove(&atom_1_old.clone());
                    bonds.entry(atom_1.clone()).insert_entry(bond_type);
                    self.adjacency_list.entry(atom_2.clone()).insert_entry(bonds);

                    // revise old bonds to show updated atom_1
                    for (atom, bonds) in filter {
                        let bond_old_type = bonds.remove(&atom_1_old).unwrap();
                        bonds.entry(atom_1.clone()).insert_entry(bond_old_type);
                        self.adjacency_list.entry(atom.clone()).insert_entry(bonds.clone());
                    }
                }
                None => {
                    self.adjacency_list.entry(atom_2.clone()).or_insert_with(HashMap::new).insert(atom_1.clone(), bond_type);
                }
            }
        }

        bond_allowed
    }

    fn contains_atom(&self, atom: &Atom) -> bool {
        self.adjacency_list.contains_key(atom)
    }

    fn get_atoms(& self) -> HashSet<&Atom> {
        self.adjacency_list.keys().collect()
    }

    fn neighbors(&self, atom: &Atom) -> Option<HashSet<&Atom>> {
        let set : HashSet<&Atom> = self.adjacency_list
            .get(atom)
            .unwrap()
            .keys()
            .collect();
        if set.is_empty() {
            None
        } else {
            Some(set)
        }
    }

    fn used_electrons_total(&self) -> i64 {
        self.adjacency_list.iter()
            .map(|(atom, map)| {
                atom.get_bonded_electrons_count(map) + atom.lone
            }).sum()
    }

    fn print_graph(&self) {
        for (atom, neighbors) in &self.adjacency_list {
            println!("{:?} -> {:?}", atom, neighbors);
        }
    }
}

type AtomRef = Rc<RefCell<Atom>>;

#[derive(Debug, Clone)]
struct Model {
    atoms: Vec<AtomRef>,
    bonds_with: Vec<Vec<(AtomRef, BondType)>>,
}

impl Model {
    fn new(atoms: Vec<AtomRef>) -> Self {
        Model {
            atoms: atoms.clone(),
            bonds_with: vec![Vec::new(); atoms.len()],
        }
    }

    fn add_bond(&mut self, atom_1: AtomRef, atom_2: AtomRef, bond: BondType) {

        let mut index_of_atom_1 = 0;
        let mut index_of_atom_2 = 0;
        for i in 0..self.atoms.len() {
            if self.atoms[i] == atom_1 {
                index_of_atom_1 = i;
            }
            if self.atoms[i] == atom_2 {
                index_of_atom_2 = i;
            }
        }

        let mut bonds_of_atom_1 = self.bonds_with.remove(index_of_atom_1);
        bonds_of_atom_1.push((atom_2, bond));
        self.bonds_with.insert(index_of_atom_1, bonds_of_atom_1);

        let mut bonds_of_atom_2 = self.bonds_with.remove(index_of_atom_2);
        bonds_of_atom_2.push((atom_1, bond));
        self.bonds_with.insert(index_of_atom_2, bonds_of_atom_2);
    }
}


fn main() {

    let valences: HashMap<&str, i64> = HashMap::from([
        ("H", 1),  // Hydrogen
        ("He", 2),  // Helium
        ("Li", 1),  // Lithium
        ("Be", 2),  // Beryllium
        ("B", 3),  // Boron
        ("C", 4),  // Carbon
        ("N", 5),  // Nitrogen
        ("O", 6),  // Oxygen
        ("F", 7),  // Fluorine
        ("Ne", 8),  // Neon
        ("Na", 1),  // Sodium
        ("Mg", 2),  // Magnesium
        ("Al", 3),  // Aluminum
        ("Si", 4),  // Silicon
        ("P", 5),  // Phosphorus
        ("S", 6),  // Sulfur
        ("Cl", 7),  // Chlorine
        ("Ar", 8),  // Argon
        ("K", 1),  // Potassium
        ("Ca", 2),  // Calcium
        ("Ga", 3),  // Gallium
        ("Ge", 4),  // Germanium
        ("As", 5),  // Arsenic
        ("Se", 6),  // Selenium
        ("Br", 7),  // Bromine
        ("Kr", 8),  // Krypton
        ("Rb", 1),  // Rubidium
        ("Sr", 2),  // Strontium
        ("In", 3),  // Indium
        ("Sn", 4),  // Tin
        ("Sb", 5),  // Antimony
        ("Te", 6),  // Tellurium
        ("I", 7),  // Iodine
        ("Xe", 8),  // Xenon
        ("Cs", 1),  // Cesium
        ("Ba", 2),  // Barium
        ("Tl", 3),  // Thallium
        ("Pb", 4),  // Lead
        ("Bi", 5),  // Bismuth
        ("Po", 6),  // Polonium
        ("At", 7),  // Astatine
        ("Rn", 8),  // Radon
    ]);

    let electronegativities: HashMap<&str, i64> = HashMap::from([
        ("H", 220),
        ("He", 0), // Helium has no electronegativity.
        ("Li", 98),
        ("Be", 155),
        ("B", 204),
        ("C", 255),
        ("N", 304),
        ("O", 343),
        ("F", 398),
        ("Ne", 0), // Neon has no electronegativity.
        ("Na", 93),
        ("Mg", 143),
        ("Al", 161),
        ("Si", 190),
        ("P", 219),
        ("S", 258),
        ("Cl", 316),
        ("Ar", 0), // Argon has no electronegativity.
        ("K", 82),
        ("Ca", 100),
        ("Ga", 181),
        ("Ge", 200),
        ("As", 220),
        ("Se", 240),
        ("Br", 280),
        ("Kr", 0), // Krypton has negligible electronegativity.
        ("Rb", 82),
        ("Sr", 95),
        ("In", 160),
        ("Sn", 199),
        ("Sb", 220),
        ("Te", 221),
        ("I", 259),
        ("Xe", 0), // Xenon has negligible electronegativity.
        ("Cs", 79),
        ("Ba", 89),
        ("Tl", 190),
        ("Pb", 200),
        ("Bi", 207),
        ("Po", 220),
        ("At", 270),
        ("Rn", 0), // Radon has negligible electronegativity.
        ("Fr", 70),
        ("Ra", 90),
    ]);

    let args: Vec<String> = env::args().collect();
    let input_compound = parse_input(&args, &valences, &electronegativities);

    let model_compound = test(&input_compound);
    dbg!(&model_compound);
}

fn parse_input(_args : &[String], valences: &HashMap<&str, i64>, electronegativities: &HashMap<&str, i64>) -> ParsedCompound {
    let charge: i64 = _args[2].clone().parse().unwrap();

    let inputted_compound = _args[1].clone();

    let mut counts: Vec<i64> = vec![];

    let mut element_names: Vec<String> = vec![];
    let mut element_name = String::new();

    for (i, c) in inputted_compound.chars().enumerate() {
        if c.is_alphabetic() {
            if c.is_uppercase() {
                if i != 0 && inputted_compound.chars().nth(i-1).unwrap().is_alphabetic() {
                    counts.push(1);
                }
                if !element_name.is_empty() {
                    element_names.push(element_name.clone());
                    element_name.clear();
                }
            }
            element_name.push(c);
        } else if c.is_digit(10) {
            if !element_name.is_empty() {
                element_names.push(element_name.clone());
                element_name.clear();
            }
            counts.push(c.to_digit(10).unwrap() as i64);
        }
    }
    if !element_name.is_empty() {
        element_names.push(element_name);
        if inputted_compound.chars().nth(inputted_compound.chars().count() - 1).unwrap().is_alphabetic() {
            counts.push(1);
        }
    }

    let mut elements: HashSet<Element> = HashSet::new();

    let zipped = zip(element_names.iter(), counts.iter());
    zipped.for_each(|(element, count)|
        for i in 0..*count {
            let ele = Element {
                name: element.clone(),
                valence: *valences.get(element.as_str()).unwrap(),
                electroneg: *electronegativities.get(element.as_str()).unwrap(),
                id: i
            };
            elements.insert(ele);
        }
    );

    ParsedCompound { elements, charge }
}

fn build_model_compound(input_compound: &ParsedCompound) -> Option<ModelCompound> {
    let mut compound = ModelCompound::new();
    let charge = input_compound.charge;

    // count total num of valence electrons
    let valence_electron_count: i64 = input_compound.elements
        .iter()
        .map(|element| element.valence)
        .sum();

    // determine least electronegative atom
    let min_electroneg = input_compound.elements.iter().min_by(|a, b| a.electroneg.cmp(&b.electroneg)).unwrap();
    let mut min_atom : Atom = Atom {name: String::new(), valence: 0, lone: 0, id: 0};

    // add all atoms to compound
    input_compound.elements.iter().for_each(|element| {
        let atom : Atom = Atom { name: element.name.clone(), valence: element.valence, lone: element.valence, id: element.id };
        compound.add_atom(&atom);
        if min_electroneg.name == element.name && min_electroneg.id == element.id {
            min_atom = atom;
        }
    });

    compound.clone().adjacency_list.iter().for_each(|(atom, _)| {
        if atom.name == min_atom.name && atom.id == min_atom.id {
            return;
        } else {
            compound.add_bond(atom.name.as_str(), atom.id, min_atom.name.as_str(), min_atom.id, BondType::SINGLE);

        }
    });

    dbg!(&compound.print_graph());

    // determine bonds

    let mut queue: VecDeque<ModelCompound> = VecDeque::new();
    queue.push_front(compound);
    while !queue.is_empty() {
        let curr = queue.pop_front().unwrap();
        dbg!(&curr.print_graph());
        let remaining_electrons = valence_electron_count % 8;
        let octet_electrons = valence_electron_count / 8;

        if charge == 0 {
            let mut electrons_to_full_map: HashMap<&Atom, i64> = HashMap::new();
            curr.adjacency_list.iter().for_each(|(atom, bonds)| {
                if atom.name == "H" || atom.name == "He" {
                    electrons_to_full_map.entry(atom).insert_entry(atom.electrons_to_two(bonds));
                } else {
                    electrons_to_full_map.entry(atom).insert_entry(atom.electrons_to_octet(bonds));
                }
            });
            if electrons_to_full_map.values().sum::<i64>() == 0 && curr.adjacency_list.iter().map(|(atom, bonds)| {atom.formal_charge(bonds)}).sum::<i64>() == 0 {
                return Some(curr);
            } else {
                curr.clone().adjacency_list.iter().for_each(|(atom, bonds)| {
                    let mut curr_cloned_1 = curr.clone();
                    // if bond does not exist yet
                    curr_cloned_1.clone().get_atoms().iter().for_each(|present_atom| {
                        match bonds.get(present_atom) {
                            None => {
                                if atom.name == present_atom.name && atom.id == present_atom.id {
                                    return;
                                } else {
                                    let mut curr_cloned_2 = curr_cloned_1.clone();
                                    //curr_cloned_2.remove_bond(atom.name.as_str(), atom.id, present_atom.name.as_str(), present_atom.id);
                                    curr_cloned_2.add_bond(atom.name.as_str(), atom.id, present_atom.name.as_str(), present_atom.id, BondType::SINGLE);
                                    queue.push_front(curr_cloned_2.clone());
                                }
                            }
                            Some(bond) => {
                                let mut curr_cloned_2 = curr_cloned_1.clone();
                                if atom.name == present_atom.name && atom.id == present_atom.id {
                                    return;
                                } else {
                                    match bond {
                                        BondType::SINGLE => {
                                            //curr_cloned_2.remove_bond(atom.name.as_str(), atom.id, present_atom.name.as_str(), present_atom.id);
                                            curr_cloned_2.add_bond(atom.name.as_str(), atom.id, present_atom.name.as_str(), present_atom.id, BondType::DOUBLE);
                                            queue.push_front(curr_cloned_2.clone());
                                        }
                                        BondType::DOUBLE => {
                                            //curr_cloned_2.remove_bond(atom.name.as_str(), atom.id, present_atom.name.as_str(), present_atom.id);
                                            curr_cloned_2.add_bond(atom.name.as_str(), atom.id, present_atom.name.as_str(), present_atom.id, BondType::TRIPLE);
                                            queue.push_front(curr_cloned_2.clone());
                                        }
                                        BondType::TRIPLE => {}
                                    }
                                }
                            }
                        }
                    })
                })
            }
        } else {

        }
    }

    None
}

fn test(input_compound: &ParsedCompound) -> Model {

    let min_electroneg = input_compound.elements.iter().filter(|x| x.name != "H").min_by(|a, b| a.electroneg.cmp(&b.electroneg)).unwrap();
    let mut min_atom: AtomRef = Atom::new("", 0, 0, 0);
    let mut atoms_vec: Vec<AtomRef> = Vec::new();

    // add all atoms to compound
    input_compound.elements.iter().for_each(|element| {
        let atom = Atom { name: element.name.clone(), valence: element.valence, lone: element.valence, id: element.id };
        let atomref = Rc::new(RefCell::new(atom.clone()));
        atoms_vec.push(atomref.clone());
        if min_electroneg.name == element.name && min_electroneg.id == element.id {
            min_atom = atomref.clone();
        }
    });

    dbg!(&atoms_vec.len());

    let mut compound = Model::new(atoms_vec);


    let atoms = compound.atoms.clone();

    for i in atoms.iter() {
        if *i == min_atom.clone() {
            continue;
        } else {
            compound.add_bond(i.clone(), min_atom.clone(), BondType::TRIPLE);
        }
    }

    // compound.clone().adjacency_list.iter().for_each(|(x, y)| {
    //     dbg!(&x.name, &x.id);
    //     dbg!(&x.electrons_to_octet(y));
    //     dbg!(&x.formal_charge(y));
    // });

    compound
}

/*
first count the total number of valence electrons in the molecule,
then arrange the atoms with the least electronegative atom as the central atom,
and distribute the electrons around each atom to satisfy the octet rule
 by placing bonding pairs between atoms and lone pairs on individual atoms,
 while minimizing formal charges where possible
 */