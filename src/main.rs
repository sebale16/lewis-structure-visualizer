use std::env;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::zip;

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
    fn get_bonded_electrons_count(&self, bonds_with: &HashMap<Atom, BondType>) -> i64 {
        bonds_with.iter().map(|(atom, bond_type)| {
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
}

#[derive(Clone, Copy, Debug)]
enum BondType {
    SINGLE,
    DOUBLE,
    TRIPLE
}

struct ParsedCompound {
    elements: HashSet<Element>, // vector corresponding to each element
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

    fn add_bond(&mut self, atom_1: &Atom, atom_2: &mut Atom, bond_type: BondType) -> bool {
        let mut atom_1_new = atom_1.clone();
        let mut atom_2_old = atom_2.clone();

        let bonds_1_or_none = self.adjacency_list.remove(&atom_1);
        let bonds_2_or_none = self.adjacency_list.remove(&atom_2_old);

        let mut bond_allowed = false;

        match bond_type {
            BondType::SINGLE => {
                if atom_1.lone > 0 && atom_2.lone > 0 {
                    bond_allowed = true;
                    atom_1_new.lone -= 1;
                    atom_2.lone -= 1;
                    dbg!(&atom_2);
                }
            }
            BondType::DOUBLE => {
                if atom_1.lone > 1 && atom_2.lone > 1 {
                    bond_allowed = true;
                    atom_1_new.lone -= 2;
                    atom_2.lone -= 2;
                }
            }
            BondType::TRIPLE => {
                if atom_1.lone > 2 && atom_2.lone > 2 {
                    bond_allowed = true;
                    atom_1_new.lone -= 3;
                    atom_2.lone -= 3;
                }
            }
        }

        if bond_allowed {
            match bonds_1_or_none {
                Some(mut bonds) => {
                    bonds.remove(&atom_2_old.clone());
                    bonds.entry(atom_2.clone()).insert_entry(bond_type);
                    self.adjacency_list.entry(atom_1_new.clone()).insert_entry(bonds);
                }
                None => {
                    self.adjacency_list.entry(atom_1_new.clone()).or_insert_with(HashMap::new).insert(atom_2.clone(), bond_type);
                }
            }

            match bonds_2_or_none {
                Some(mut bonds) => {
                    bonds.remove(atom_1);
                    bonds.entry(atom_1_new.clone()).insert_entry(bond_type);
                    self.adjacency_list.entry(atom_2.clone()).insert_entry(bonds);
                }
                None => {
                    self.adjacency_list.entry(atom_2.clone()).or_insert_with(HashMap::new).insert(atom_1_new.clone(), bond_type);
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
    dbg!(model_compound.used_electrons_total());
    dbg!(model_compound.print_graph());
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

    // count total num of valence electrons
    let valence_electron_count: i64 = input_compound.elements
        .iter()
        .map(|element| element.valence)
        .sum();

    // determine least electronegative atom
    let min_electroneg = input_compound.elements.iter().min_by(|a, b| a.electroneg.cmp(&b.electroneg)).unwrap();
    let mut min_atom : Atom = Atom {name: String::new(), valence: 0, lone: 0, id: 0};

    dbg!(valence_electron_count);

    // add all atoms to compound
    input_compound.elements.iter().for_each(|element| {
        let atom : Atom = Atom { name: element.name.clone(), valence: element.valence, lone: element.valence, id: element.id };
        compound.add_atom(&atom);
        if min_electroneg.name == element.name && min_electroneg.id == element.id {
            min_atom = atom;
        }
    });

    compound.get_atoms().iter().for_each(|atom| {

    });

    // determine bonds
    let formal_charge_total = 8;

    let mut queue: VecDeque<ModelCompound> = VecDeque::new();
    queue.push_front(compound);
    while !queue.is_empty() {
        let curr = queue.pop_front().unwrap();

        let mut curr_formal_charge_total = 0;

        if curr_formal_charge_total == formal_charge_total {
            return Some(curr);
        } else {
            let curr_used_electrons_total = curr.used_electrons_total();
            if curr_used_electrons_total < valence_electron_count {

            }
        }
    }

    None
}

fn test(input_compound: &ParsedCompound) -> ModelCompound {
    let mut compound = ModelCompound::new();

    let min_electroneg = input_compound.elements.iter().filter(|x| x.name != "H").min_by(|a, b| a.electroneg.cmp(&b.electroneg)).unwrap();
    let mut min_atom : Atom = Atom {name: String::new(), valence: 0, lone: 0, id: 0};


    // add all atoms to compound
    input_compound.elements.iter().for_each(|element| {
        let atom : Atom = Atom { name: element.name.clone(), valence: element.valence, lone: element.valence, id: element.id };
        compound.add_atom(&atom);
        if min_electroneg.name == element.name && min_electroneg.id == element.id {
            min_atom = atom;
        }
    });

    let mut compound_cloned = compound.clone();
    let mut atoms = compound_cloned.get_atoms();

    for i in atoms.iter() {
        if *i == &min_atom {
            continue;
        } else {
            compound.add_bond(&i, &mut min_atom, BondType::SINGLE);
        }
    }

    compound
}

/*
first count the total number of valence electrons in the molecule,
then arrange the atoms with the least electronegative atom as the central atom,
and distribute the electrons around each atom to satisfy the octet rule
 by placing bonding pairs between atoms and lone pairs on individual atoms,
 while minimizing formal charges where possible
 */